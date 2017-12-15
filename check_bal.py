# < begin copyright > 
# Copyright Ryan Marcus 2017
# 
# This file is part of basicaf.
# 
# basicaf is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
# 
# basicaf is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
# 
# You should have received a copy of the GNU General Public License
# along with basicaf.  If not, see <http://www.gnu.org/licenses/>.
# 
# < end copyright > 
 
import sys

with open(sys.argv[1], "r") as f:
    bf = f.read()


def count_braces(x):
    num_left = 0
    num_right = 0

    for c in x:
        if c == "<":
            num_left += 1
        elif c == ">":
            num_right += 1

    return num_left, num_right

def brace_pairs(x):
    first_lb = x.find("[")
    if first_lb == -1:
        return

    num_open = 1
    for i in range(first_lb+1, len(x)):
        if x[i] == "[":
            num_open += 1
        elif x[i] == "]":
            num_open -= 1
            if num_open == 0:
                yield x[first_lb + 1:i]
                yield from brace_pairs(x[first_lb + 1:i - 1])
                yield from brace_pairs(x[i + 1:])
                return
            
for bp in brace_pairs(bf):
    print("{", bp, "}")
    print(count_braces(bp))
                    
        
    
        

