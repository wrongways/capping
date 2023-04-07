"""Utility class to simplify access to configuration properties"""

from configparser import ConfigParser
from pathlib import Path
import sys


class Config:
    """Transforms the config file into easy to use attributes"""

    def __init__(self, config_filename) -> None:
        config_path = Path(config_filename)
        if not (config_path.exists() and config_path.is_file()):
            bel = "\07"
            sys.exit(f"File {config_path} does not exist{bel}")

        config = ConfigParser()
        config.read(config_path)
        self.config = {section.lower(): dict(config[section]) for section in config.sections()}

    @property
    def bmc(self):
        return self.config["bmc"]

    @property
    def bmc_user(self):
        return self.bmc["user"]

    @property
    def bmc_password(self):
        return self.bmc["password"]

    @property
    def bmc_name(self):
        return self.bmc["name"]
