from PIL import Image

width: int = 6
height: int  = 5
rot: int = 4


img = Image.new("RGB", (width * rot, height))
canvas = img.load()

for x in range(width * rot):
    for y in range(height):
        canvas[x, y] = (
            int(float(x % width) / width * 255),
            int(float(y) / height * 255),
            int(float(x // width) / rot  * 255)
        )

img.save("map.png")