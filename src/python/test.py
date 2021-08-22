from PIL import Image
import json
from numpy import asarray
import lenna_cli

image = Image.open('lenna.png')
data = asarray(image)
print(data.shape)

lennaCli = lenna_cli.LennaCli()
lennaCli.load_plugins("plugins")
print(lennaCli.plugins())

config = json.loads('{"pipeline": [{"id": "resize","width": 200,"height": 200}]}')

processed = lennaCli.process(config, data)
print(processed.shape)
Image.fromarray(processed).save('lenna_test_out.png')
