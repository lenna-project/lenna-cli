from PIL import Image
from numpy import asarray
import lenna_cli

lennaCli = lenna_cli.LennaCli()
lennaCli.load_plugins("plugins")
print(lennaCli.plugins())
