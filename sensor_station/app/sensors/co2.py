import random


class CO2SensorEmulator:
    def __init__(self, base_ppm=400, noise_level=10):
        """
        Initialize the CO2 sensor emulator.
        :param base_ppm: The base CO2 level in ppm (default: 400 ppm).
        :param noise_level: The maximum deviation due to noise (default: 10 ppm).
        """
        self.base_ppm = base_ppm
        self.noise_level = noise_level

    def read_co2(self) -> int:
        """
        Simulate reading the CO2 level with noise.
        :return: Simulated CO2 ppm value.
        """
        noise = random.uniform(-self.noise_level, self.noise_level)
        return int(round(self.base_ppm + noise, 2))


# Example usage
if __name__ == "__main__":
    sensor = CO2SensorEmulator()
    while True:
        co2 = sensor.read_co2()
        print(co2)
