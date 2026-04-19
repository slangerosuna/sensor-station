import random


class PressureSensorEmulator:
    def __init__(self, base_pa=101325, noise_level=1000):
        """
        Initialize the Pa sensor emulator.
        :param base_pa: The base pressure level in Pa (default: 400 Pa).
        :param noise_level: The maximum deviation due to noise (default: 10 ppm).
        """
        self.base_pa = base_pa
        self.noise_level = noise_level

    def read_pa(self) -> float:
        """
        Simulate reading the Pa level with noise.
        :return: Simulated Pa ppm value.
        """
        noise = random.uniform(-self.noise_level, self.noise_level)
        return round(self.base_pa + noise, 2)


# Example usage
if __name__ == "__main__":
    sensor = PressureSensorEmulator()
    while True:
        pa = sensor.read_pa()
        print(pa)
