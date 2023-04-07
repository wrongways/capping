from pathlib import Path
from .config import Config


class Collector:
    def __init__(self) -> None:
        current_directory = Path(__file__).parent
        config_file_path = Path(current_directory, '../config/config.ini')
        self.config = Config(config_file_path)
        print(self.config.bmc_user)


    @property
    def sensors(self):
