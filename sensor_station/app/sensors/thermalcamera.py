import numpy as np


class ThermalCameraSensorEmulator:
    def __init__(self, base_c=13, noise_level=3, image_size=(24, 32)):
        """
        Initialize the thermal camera emulator.
        :param base_c: The base temp level in C (default: 13 C).
        :param noise_level: The maximum deviation due to noise (default: 3 C).
        """
        self.base_c = base_c
        self.noise_level = noise_level
        self.image_size = image_size

    def read_image(self) -> np.ndarray:
        """
        Simulate reading the thermal image with noise.
        :return: Simulated thermal image.
        """
        noise = np.random.uniform(-self.noise_level, self.noise_level, self.image_size)
        return np.round(self.base_c + noise, 2)


# Example usage
if __name__ == "__main__":
    sensor = ThermalCameraSensorEmulator()
    while True:
        temp_image = sensor.read_image()
        print(temp_image)
