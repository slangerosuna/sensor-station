import random


class AltitudeSensorEmulator:
    def __init__(self, base_alt=171, noise_level=5):
        """
        Initialize the altitude sensor emulator.
        :param base_alt: The base altitude in m (default: 171m).
        :param noise_level: The maximum deviation due to noise (default: 5m).
        """
        self.base_alt = base_alt
        self.noise_level = noise_level

    def read_alt(self) -> float:
        """
        Simulate reading the altitude level with noise.
        :return: Simulated altitude value.
        """
        noise = random.uniform(-self.noise_level, self.noise_level)
        return round(self.base_alt + noise, 2)


# Example usage
if __name__ == "__main__":
    sensor = AltitudeSensorEmulator()
    while True:
        alt = sensor.read_alt()
        print(alt)
