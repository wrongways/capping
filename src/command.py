from subprocess import Popen
import shlex
import subprocess
import sys

from config import Config
conf = Config()
auth_string = f"-h {conf.bmc_name} -u {conf.bmc_user} -p {conf.bmc_password}"

command = f"/usr/sbin/ipmi-sensors " + auth_string
print(command)

command_tokens = shlex.split(command)
rc = subprocess.run(command_tokens, capture_output=True)
if rc.returncode != 0:
    sys.exit(f"{command} failed: {rc.stderr.decode()}")

sensor_list = rc.stdout.decode()
for line in sensor_list:
    print(line)

