import sys
import pyintedit
a = -17080198121677823
a = a - 1
b = a * 10
c = pyintedit.testmyshit(a)
print(f'{b == c = }')
print(sys.getrefcount(a))
print(sys.getrefcount(c))


