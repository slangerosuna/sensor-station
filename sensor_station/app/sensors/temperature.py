import random


class TempSensorEmulator:
    def __init__(self, base_temp=15, noise_level=3):
        """
        Initialize the temperature sensor emulator.
        :param base_temp: The base temp in C (default: 15C).
        :param noise_level: The maximum deviation due to noise (default: 3 C).
        """
        self.base_temp = base_temp
        self.noise_level = noise_level

    def read_temp(self) -> float:
        """
        Simulate reading the temperature level with noise.
        :return: Simulated temp C value.
        """
        noise = random.uniform(-self.noise_level, self.noise_level)
        return round(self.base_temp + noise, 2)


# Example usage
if __name__ == "__main__":
    sensor = TempSensorEmulator()
    while True:
        temp = sensor.read_temp()
        print(temp)
