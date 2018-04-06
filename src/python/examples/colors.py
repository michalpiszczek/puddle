import networkx as nx

from puddle import Session, mk_session, Droplet
from examples.dilution import dilute, VolConcDroplet


def blend(session, d_black_factory, d_red_factory, d_green_factory,
            d_blue_factory, rgb_target, epsilon=0.001):

    red_target = rgb_target[0]
    green_target = rgb_target[1]
    blue_target = rgb_target[2]

    red_droplet = dilute(session, d_black_factory, d_red_factory,
            red_target, epsilon=epsilon)

    red_droplet.rgb = [red_droplet.concentration, 0, 0]

    print("RED GOOD")
    assert red_droplet.volume == 1
    print("RED GOODER")

    green_droplet = dilute(session, d_black_factory, d_green_factory,
            green_target, epsilon=epsilon)

    assert green_droplet.volume == 1

    green_droplet.rgb = [0, green_droplet.concentration, 0]

    blue_droplet = dilute(session, d_black_factory, d_blue_factory,
            blue_target, epsilon=epsilon)

    assert blue_droplet.volume == 1

    blue_droplet.rgb = [0, 0, blue_droplet.concentration]

    result = session.mix(red_droplet, session.mix(green_droplet, blue_droplet))

    result.rgb[0] = red_droplet.rgb[0]
    result.rgb[1] = green_droplet.rgb[1]
    result.rgb[2] = blue_droplet.rgb[2]

    return result


class RGBDroplet(VolConcDroplet):

    def __init__(self, *args, **kwargs):
        self.rgb = kwargs.pop('rgb', [0.0, 0.0, 0.0])
        print(kwargs)
        super().__init__(*args, **kwargs)

    def mix(self, other):
        result = super().mix(other)
        result.rgb = [0.0, 0.0, 0.0]
        for i in range(3):
            mass1 = self.rgb[i] * self.volume
            mass2 = other.rgb[i] * other.volume
            result.rgb[i] = (mass1 + mass2) / result.volume
        return result

    def split(self):
        d1, d2 = super().split()
        d1.rgb = self.rgb
        d2.rgb = self.rgb
        return d1, d2

with mk_session('../../tests/arches/arch-big.json') as session:

    rgb_black = [0.0, 0.0, 0.0]

    rgb_red = [1.0, 0.0, 0.0]
    rgb_green = [0.0, 1.0, 0.0]
    rgb_blue = [0.0, 0.0, 1.0]

    # approximately UW Purple
    rgb_target = [0.2, 0, 0.44]

    eps = 0.01

    def d_black_factory():
        return session.input(
            location=None,
            volume=1,
            concentration=0,
            rgb=rgb_black,
            droplet_class=RGBDroplet,
        )

    def d_red_factory():
        return session.input(
            location=None,
            volume=1,
            concentration=1,
            rgb=rgb_red,
            droplet_class=RGBDroplet,
        )

    def d_green_factory():
        return session.input(
            location=None,
            volume=1,
            concentration=1,
            rgb=rgb_green,
            droplet_class=RGBDroplet,
        )

    def d_blue_factory():
        return session.input(
            location=None,
            volume=1,
            concentration=1,
            rgb=rgb_blue,
            droplet_class=RGBDroplet,
        )

    d = blend(session, d_black_factory, d_red_factory, d_green_factory,
                d_blue_factory, rgb_target, epsilon=eps)

    assert abs(d.rgb[0] - rgb_target[0]) < eps
    assert abs(d.rgb[1] - rgb_target[1]) < eps
    assert abs(d.rgb[2] - rgb_target[2]) < eps

