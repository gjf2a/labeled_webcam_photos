import unittest
from std_msgs.msg import String
import sys, curses, math
import rclpy
from rclpy.node import Node
from rclpy.qos import qos_profile_sensor_data
from geometry_msgs.msg import TwistStamped


class FloorRegion:
    def __init__(self, x_start: int, length: int):
        self.x_start = x_start
        self.length = length

    def __repr__(self):
        return f"FloorRegion({self.x_start}, {self.length})"

    def __eq__(self, other):
        return self.x_start == other.x_start and self.length == other.length

    def center(self) -> int:
        return self.x_start + self.length // 2

    
def find_floor_regions(heights: list[int], min_height: int, min_width: int) -> list[FloorRegion]:
    current = None
    regions = []
    for i, h in enumerate(heights):
        if current is None and h >= min_height:
            current = i
        elif current is not None and (h < min_height or i == len(heights) - 1):
            length = i - current
            if length >= min_width:
                regions.append(FloorRegion(current, length))
            current = None
    return regions


class GroundlineTest(unittest.TestCase):
    def test_find_floor(self):
        line1 = [120, 39, 40, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 40, 40, 14, 13, 12, 9, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 8, 8, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 4, 4, 5, 5, 5, 5, 39, 40, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 40, 40, 5, 5, 5, 5, 39, 40, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39]
        regions1 = find_floor_regions(line1, 30, 16)
        self.assertEqual([FloorRegion(0, 17), FloorRegion(123, 18)], regions1)
        self.assertEqual([8, 132], [region.center() for region in regions1])


class GroundlineNode(Node):
    def __init__(self, robot_name: str, screen_width: int, min_width: int, min_height: int):
        super().__init__(f"{robot_name}_GroundlineNode")
        self.create_subscription(String, f"{robot_name}_groundline", self.groundline_callback, qos_profile_sensor_data)
        self.motors = self.create_publisher(TwistStamped, f"{robot_name}/cmd_vel_stamped", qos_profile_sensor_data)
        self.last_x = None
        self.screen_width = screen_width
        self.min_width = min_width
        self.min_height = min_height

    def groundline_callback(self, groundline_str: String):
        groundline = eval(groundline_str.data)
        regions = find_floor_regions(groundline, self.min_height, self.min_width)
        if self.last_x is None:
            if len(regions) > 0:
                self.last_x = regions[0].center()
        else:
            closest = 0
            for i, region in enumerate(regions):
                if abs(self.last_x - region.center()) < abs(self.last_x - regions[closest].center()):
                    closest = i
            self.last_x = regions[closest].center()

        t = TwistStamped()
        t.header.frame_id = "base_link"
        t.header.stamp = self.get_clock().now().to_msg()        
        if self.last_x is None:
            
            

def main(stdscr):
    rclpy.init()
    

    rclpy.shutdown()
    node.destroy_node()
    curses.nocbreak()
    curses.echo()
    stdscr.refresh()


if __name__ == '__main__':
    if len(sys.argv) == 2 and sys.argv[1] == '--test':
        unittest.main()
    elif len(sys.argv) < 5:
        print("Usage: python3 groundline.py (--test | robot_name screen_width min_width min_height)")
    else:
        curses.wrapper(main)
