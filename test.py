import sys
import pyintedit
a = -17080198121677823
a = a - 1
b = a + 1
c = pyintedit.testmyshit(a)
print(f'{b == c = }')
print(sys.getrefcount(a))
print(sys.getrefcount(c))

ol_korect = []
for a in range(-4_294_967_300, -4_294_967_000):
    correct = a+1
    candidate = pyintedit.testmyshit(a)
    ol_korect.append(candidate == correct)
print(f'negative: {all(ol_korect) = }')
ol_korect = []
for a in range(1000):
    correct = a+1
    candidate = pyintedit.testmyshit(a)
    ol_korect.append(candidate == correct)
print(f'positive: {all(ol_korect) = }')
