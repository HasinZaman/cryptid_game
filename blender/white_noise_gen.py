from PIL import Image
import random as r
width: int = 256
height: int  = 256


img = Image.new("RGB", (width, height))
canvas = img.load()

for x in range(width):
    for y in range(height):
        val = r.random()
        canvas[x, y] = (
            int(val * 255),
            int(val * 255),
            int(val  * 255)
        )

img.save("noise.png")