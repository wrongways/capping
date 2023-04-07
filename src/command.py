from pickle import TRUE
from subprocess import Popen
import shlex
import subprocess
import sys

from .config import Config
auth_string = f"-h {Config.bmc_name} -u {Config.bmc_user} -p {Config.bmc_password}"

command = (f"/usr/sbin/ipmi-sensors " + auth_string)
command_tokens = shlex.split(command)
rc = subprocess.run(command_tokens, capture_output=True)
if rc.returncode != 0:
    sys.exit(f"{command} failed: {rc.stderr}")

print(rc.stdout)

