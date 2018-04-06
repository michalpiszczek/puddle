import numpy as np
import matplotlib.pyplot as plt

from puddle import Session, mk_session, Droplet
from examples.dilution import dilute, VolConcDroplet


class ErrorDroplet(VolConcDroplet):
    """Splits with an error rate drawn from the specified distribution."""

    def __init__(self, *args, **kwargs):
        self.volume = kwargs.pop('volume', 1)
        self.concentration = kwargs.pop('concentration', 0)

        def no_error_factory():
            return 1.0

        self.error_factory = kwargs.pop('error_factory', no_error_factory)

        print(kwargs)
        super().__init__(*args, **kwargs)

    def mix(self, other):
        result = super().mix(other)
        result.error_factory = self.error_factory
        return result

    def split(self):
        d1, d2 = super().split()
        error = self.error_factory()
        d1.volume = d1.volume * error
        d2.volume = d2.volume * (2 - error)
        d1.error_factory = self.error_factory
        d2.error_factory = self.error_factory
        return d1, d2

with mk_session('../../tests/arches/arch-big.json') as session:

    c_low = 0
    c_high = 1

    c_target = .37
    eps = 0.1

    # Parameters for the Gaussian distribution
    # from which we draw Split error
    mean = 1.0
    standard_deviation = 0.001

    # Standard Gaussian distribution
    def normal_error_factory():
        return np.random.normal(mean, standard_deviation, 1)[0]

    def d_low_factory():
        return session.input(
            location=None,
            volume=1,
            concentration=c_low,
            error_factory=normal_error_factory,
            droplet_class=ErrorDroplet,
        )

    def d_high_factory():
        return session.input(
            location=None,
            volume=1,
            concentration=c_high,
            error_factory=normal_error_factory,
            droplet_class=ErrorDroplet,
        )

    d = dilute(session, d_low_factory, d_high_factory,
               c_target, epsilon=eps)

    assert abs(d.concentration - c_target) < eps
