import random


class HumiditySensorEmulator:
    def __init__(self, base_rh=30, noise_level=10):
        """
        Initialize the humidity sensor emulator.
        :param base_rh: The base relative humdity (rh) level in % (default: 30%).
        :param noise_level: The maximum deviation due to noise (default: 10%).
        """
        self.base_rh = base_rh
        self.noise_level = noise_level

    def read_rh(self) -> float:
        """
        Simulate reading the % with noise.
        :return: Simulated rh ppm value.
        """
        noise = random.uniform(-self.noise_level, self.noise_level)
        return max(0.0, round(self.base_rh + noise, 2))


# Example usage
if __name__ == "__main__":
    sensor = HumiditySensorEmulator()
    while True:
        rh = sensor.read_rh()
        print(rh)
