"""Deterministic system solver requests appended after the scalar corpus."""
import random


def append_system_requests(rng: random.Random, requests: list[str]) -> None:
    """Preserve corpus order and random state; append_system_requests(rng, requests)."""
    for case in range(256):
        requests.append('modern_sys '+' '.join(map(str,[case%7,rng.choice([1,2,3]),rng.choice([0,0,1,3,7,20]),
            rng.uniform(0.1,3.0),rng.choice([1.0,1e-150,1e150,100.0]),rng.uniform(-2.0,3.0),rng.uniform(-2.0,3.0),
            rng.choice([0.0,1e-10,0.1]),rng.choice([1e-12,0.01]),rng.choice([1e-12,0.1]),rng.choice([0,1,4,30]),
            rng.choice([-10.0,0.0,2.0]),rng.choice([-1.0,0.0,10.0])])))
    for case in range(256):
        requests.append('modern_lm '+' '.join(map(str,[case%7,rng.choice([1,2,3]),rng.choice([0,0,1,3,6,12]),
            rng.uniform(0.1,3.0),rng.choice([1.0,1e-150,1e150,100.0]),rng.uniform(-2.0,3.0),rng.uniform(-2.0,3.0),
            rng.choice([0.0,1e-10,1e-15]),rng.choice([1e-12,0.1]),rng.choice([1e-12,0.1]),rng.choice([0,1,4,30]),
            rng.choice([-10.0,0.0,2.0]),rng.choice([-1.0,0.0,10.0]),rng.choice([1e-3,1.0,1e12]),
            rng.choice([2.0,10.0]),rng.choice([0.1,0.5]),1e16,1e-15,rng.choice([1e-15,1e-6]),rng.choice([0.0,1e-10,0.01]),rng.choice([2.220446049250313e-16,10.0])])))
    for case in range(512):
        requests.append('modern_small '+' '.join(map(str,[case%10,rng.choice([0,0,1,2,4,9,20]),
            rng.choice([1.0,1e-150,1e150,1e-300,1e300]),rng.uniform(-3.0,3.0),rng.uniform(-3.0,3.0),
            rng.choice([0.0,1e-200,1e-12,0.1]),rng.choice([1e-16,100.0]),rng.choice([0,1,3,30]),
            rng.choice([0.0,0.5,10.0]),case%2,case%3==0,case%4==0,rng.choice([0.0,1e-4,0.1]),
            rng.choice([-10.0,0.0,2.0]),rng.choice([-1.0,0.0,10.0])])).replace('True','1').replace('False','0'))
    for case in range(256):
        requests.append('modern_swarm '+' '.join(map(str,[case%7,rng.choice([1,2,3,40]),rng.choice([0,0,1,5,19]),
            rng.choice([0,1,6,42,4294967295]),rng.choice([0,1,4,9]),rng.choice([1,4,15]),case%3,(case//3)%3,(case//9)%2,
            rng.choice([0.0,0.7,1.5]),rng.choice([0.0,0.5,10.0]),rng.choice([0.0,1e-8,0.1]),rng.choice([0,3]),
            rng.choice([1,3,10]),rng.choice([0.0,0.5,1.0]),rng.choice([0,2]),rng.choice([0,3,20]),case%2,
            rng.choice([0.0,1.0]),case%8,rng.choice([0,1]),rng.choice([-3.0,-0.1,0.5,3.0])])) )
    for boundary in range(3):
        for init in range(3):
            for mode in range(7):
                for dimensions in [1, 3]:
                    requests.append('modern_swarm '+' '.join(map(str,[mode,dimensions,0,6,6,12,
                        boundary,init,1,1.5,10.0,1e-8,0,1,0.5,2,3,0,0.0,1,1,3.0])))
    for case in range(256):
        requests.append('modern_global '+' '.join(map(str,[case%6,rng.choice([1,2,3]),rng.choice([0,0,1,5,19]),
            rng.choice([1,6,42]),case%4,rng.choice([3,4,8]),rng.choice([0,1,3]),rng.choice([0,1,5,20]),
            rng.choice([0.0,0.8,2.0]),rng.choice([0.0,0.9,1.0,1.1]),rng.choice([0,3,20]),
            rng.choice([0.0,1e-8,0.01]),case%2,(case//2)%2])))
