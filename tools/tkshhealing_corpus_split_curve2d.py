"""2D-specific single-segment behavior and shared recursive curve splitting."""

from itertools import product
from tkshhealing_corpus_shapeconstruct import construct_curves_2d
from tkshhealing_corpus_split_curve3d import split3d_record
from tktopalgo_corpus_breplib_records import geom2d_bspline, geom2d_trimmed


def split2d_record(
    curve: list[float],
    natural: bool,
    bounds: tuple[float, float],
    operations: list[list[float]],
) -> str:
    """Reuse the dimension-independent split history encoding; split2d_record(c,False,(0,1),[[1,1]])."""
    return split3d_record(curve, natural, bounds, operations, "sh_split_curve2d")


def append_split2d_candidates(requests: list[str]) -> None:
    """Include every candidate; crashing wrapper inputs are retained separately."""
    ranges = [
        (0.0, 1.0),
        (0.1, 0.9),
        (-0.2, 1.2),
        (0.5, 0.5),
        (1.0, 0.0),
        (0.0, 1.0 + 1e-12),
    ]
    curves = [
        [offset, 0.2, *curve]
        for curve, offset in product(construct_curves_2d(), [0, 1])
    ]
    for curve, bounds, segment in product(curves, ranges, [0, 1]):
        operations = [[0, 3, 0.25, 0.5, 0.75], [1, segment], [1, segment], [2, segment]]
        single = [[1, segment], [4, 2], [1, segment]]
        requests.append(split2d_record(curve, False, bounds, operations))
        requests.append(split2d_record(curve, False, bounds, single))
    append_split2d_histories(requests, curves)
    append_split2d_knots(requests)


def append_split2d_histories(requests: list[str], curves: list[list[float]]) -> None:
    """Contrast natural ranges with public parameter mutations."""
    for curve, segment in product(curves, [0, 1]):
        natural = [[1, segment], [0, 2, 0.3, 0.7], [2, segment]]
        changed = [[3, -0.5, 1.5], [0, 3, 0.2, 0.5, 0.8], [1, segment]]
        requests.append(split2d_record(curve, True, (0.0, 1.0), natural))
        requests.append(split2d_record(curve, False, (0.0, 1.0), changed))


def append_split2d_knots(requests: list[str]) -> None:
    """Probe knot snapping and wrapper recursion without altering native ranges."""
    poles = [(0.0, 0.0), (1.0, 2.0), (2.0, 1.0), (3.0, 0.0), (4.0, 2.0)]
    spline = geom2d_bspline(2, False, poles, [(0.0, 3), (0.3, 1), (0.7, 1), (1.0, 3)])
    curves = [
        [offset, 0.2, *curve]
        for curve, offset in product([spline, geom2d_trimmed(0.1, 0.9, spline)], [0, 1])
    ]
    for curve, shift, segment in product(
        curves, [-2e-9, -1e-10, 0.0, 1e-10, 2e-9], [0, 1]
    ):
        operations = [
            [0, 2, 0.3 + shift, 0.7 + shift],
            [1, segment],
            [3, 0.2, 0.8],
            [1, segment],
        ]
        requests.append(split2d_record(curve, False, (0.0, 1.0), operations))


# OCCT 3d097a0 offset recursion dereferences native null multipart pieces.
# Retain these as native-crash/Rust-typed-failure divergences, never parity passes.
SPLIT2D_CRASH_SHA256 = frozenset(
    [
        "2d8dcee40c2caadb0ce61a17949ff858c809cc575d8e97c28f9c41f5ba09b70d",
        "b1378c3a059e9aaa416012fb8843b51089cac64b15d1e694462ed35f4cdeb071",
        "b52513840ec91bb33c0efa5d1da150c4895ecca8d59469b04a242f6cd303b222",
        "0a232d428f885b81d23cebdfdf66cf4ac79854b66ec99732a88e1fd021d8ca63",
        "2df04662f90ee419f9189514b93ca8f31931cf542ed188496a92c682e0ef54aa",
        "d48f1f04bd6c4d0371d4a4176958a574bb575bdcbd417ee449ad671a165ea18f",
        "bf2fc0841263cb721538d4b65a9d3680aae994702c25e3c4f3aa9b12d3d98f78",
        "818522b6edaf4809d335979edff157027324694265fbe12dbece4db3cbc5c459",
        "c9c26bcd78071826c05249ee1540233631a1b329f2c58c4f11a861beacb3f32f",
        "7ade202facd7358940f37fd79fc8369b13cd62eed4e3e05cb2f03a738a15e18c",
        "f7d9226ad77f65fcaea8485a13e492f66d178684c85dbdb64702eab903275198",
        "471e28006f15f5286a4eb37a8b49e1991979b7ffb751df995044fe3761ec682a",
        "2011c68e7d8e72beb9d7f9402a8cc091aaee13be6f69f37329eb0c390a450103",
        "c5113ad4c1afbb9e4456b4777927c616b4f8097e42f2f16fd9b1f46715abda82",
        "7ab85e3d2f8e4c823743b1601b964da88ba1fcf59ec7cb63de728c522a803156",
        "0f94c1654229c0c118fcb8409573a161fbb4b848b6be6c0e340cefe074b006c5",
        "65455e8f39515b41abbe4c9e806bc85ccaa35433a388129be0805b071f1c1cdd",
        "542a42b15178ead9141c8af1941280dde1dc8afdf74d037672830f8263908109",
        "5c170af57cb70724e8400517f8bf4cf04c7b265e39042502352e4b2814f82059",
        "d83b0577317b78aee1b9475958ca216747f598c9f8f7515e689d32a813ba52a1",
        "83f81020ba0779b3e1a8ea1863c0b98cd001a117a5bc10d3519c76666a145bfe",
        "9ec63f1fa3aa039e2e98e9d70cfa4ad4fffa69f17e911d85f8686f2dab2b1fc1",
        "adcf6258c761f9a773cc10a89b06c8e440ae9634114c3849f2b4e5d87af772d0",
        "c3dc9ceb52587ebb506b5d9fff784bd76e084fe7e38aae6c2681dc83fa90403f",
        "fa4fe8bc549c31005acb4d5f212357754a052c4dda378ace99d37ffa386fb460",
        "611929ec44fba8822300b7bd8ee4aa46a7e652e1fd808a8d343425eebe4e8e84",
        "85fab05f63bfa00012dcc1fc4312b97190b40435dd8547f2e4f6cad69937c633",
        "e42987f64ed753efa2f43cfdb0370e766fde524a36b3c2b8fb468416efcf46a5",
        "5fe07ee4a8af819d5b1932ccf6353b76faed3448732c62e93fe9ed71d077332c",
        "b9d928f586514b0cc216bc6b26706d43b4b5b8c4ba0d83ce588829b7340d470b",
        "f6fca74ef93f61eb08cd495a0e80e62852fac5da9e6d5ed4f7b2287650691aa7",
        "748556a83a7e4648822fac17147a40e5454ab91634c5fe28459cfab8e111b169",
        "239d4e1f90cbf50a67d3ac343b01ed76a36dd33f798cd97108b9c4ec46c029ff",
        "a516611da52aab729cc79fb24c717a081ce2626c3e97c985fd630f15320f0751",
        "16ec4b4feed094268cfab783edc07a332d1bc5a4653f400c56c6fd5940091d2f",
        "96008eb89157f7fe644ca2fa43d77d285e1484cfa29cfa72c84ce637b846679d",
        "e1a87e7e4e047d699587f4f03143d7f97f7496a0c33f7614877311f4c157eb26",
        "c85e96bc143ff99cddf515ae7de8556a94405cd4f22a37eb93b12cfbe8bcc3c1",
        "230b5d70986a869ee4ca94b0c9f7b9a1163d265941df0f8f3f8e798c46ceb180",
        "47ae05bb8bfdba03d570acfd800a0cae75328a9c99313d86c7ec6872bd5f2f47",
        "c825f4c0ae4a800b5d1bf54abb621ed7df1dc5b0dbf8ec488646d93d4ed86382",
        "615a4b600b15f9c6eec34d2e5f2bd82a1a8840595c92d226cf7323d20c030b72",
        "3d5fc3b3a0b7a1d40e55def73b4c7723e36ce99e88c1543e761242c053164bae",
        "d22d992a8ea329309377d16c8606ef045ae979c85c3dd4afc8b5a0d29e3fea19",
        "2a73b2d5a3be8d45e0c9a41e329e396039ba5da1179d994f11c22630b3574911",
        "a4cd7cd61953ca3c7252e17a0b27b3e34c40e882c218c3763baea8c2bd7d0cd2",
        "29cc74726fe91269f0d48318b3926f7e94207f286ebc39050d846032ea9cdc9b",
        "09d16b752281540ce8258993732c03f6511edf570c93b02464d1feb0731e3102",
        "65ad89e7cb19a78bcd4e9de47654820313e47ee1b09b652bbd0b46347c44120a",
        "4e03ce255e5db52f7d429161554a4757fcd6a954d9aa6e6eaf42ad41525ba5fd",
        "9b9a6cc7b295015fbeefa46bc38e74f7c93a06992a9a230a000f2adbebebd824",
        "99da22edc8b615dd6a1e4e3860c95b3851eea5de8c0477914beb581426d6cf1d",
        "1b45bcb02f0abeb6dd4443629706b2d7cb71a31b9bb77cd9e89321b08b2fc3af",
        "9227d221a1cab198d98fcd5d5adcd26b6dc03950812e39ef38d6f5e2b8852d33",
        "c201ad3f882b182ec26db84afc1f63ef66295ae04f677f922cb4c026c3685469",
        "b44430e6ec0272571e436a769696abe0489183d7051fd3798b4689582b5e5125",
        "65ee699e1d0b02e5164a8f14a0f02d13c9221c74d79a8cb0989d82bb728201ae",
        "4217ea0361f9c9b41f16b16f752d6dfaefdf27c66a55e4f70ee17ba84c2cd5d4",
        "029f95bb12803d778817280da1bc585471ae3080e23c73e30e351df9aeb4ad83",
        "611cb504ebcd4342bd8b0da4f70bc98769d2dd409904677a0bffdb9a265c7289",
        "ff293970c349bf9b918a582778e147ff22e7f1e882463bc84fc341156e803a31",
        "369a1f7926a60269e9e5b757a1790f328288f20be1fbbb223a506419b9bdadf1",
        "dd213d4aa26e9c1f7d68db28a1f434b35bf369178d324766943f3ab9286f1733",
        "968fcf08990b364c54e4cbf0ffa072baebb07ff02b06fba589f434b303640b14",
        "3c74352968fc8e28d343db838de83f7c541eb7074202ebc5e683395c056262b4",
        "c6004964fa87fc679ad7beb136d4fe7baa5c9bc63a7fc28de88c21146fba9878",
        "7260e55af6b9fbf1369e1920eb520850f994dd188c796479efdfb16db1a13fd6",
        "3953cf4809f957d5eca6e4ac49f9861bd9f21cc9dbb1e0be5234c3c713c69d1b",
        "cc802260f733d55e209341a913d33ec8d34f32cb2b957c52e353d70f9339cc12",
        "20229f7cee0f0c94ac37de299a3ed34e46f6d64a98480d12dca995238b844127",
    ]
)


def split2d_partition() -> tuple[list[str], list[str]]:
    """Account every candidate and require all recorded crash inputs to remain."""
    from native_crash_accounting import partition_crash_requests

    candidates: list[str] = []
    append_split2d_candidates(candidates)
    return partition_crash_requests(candidates, SPLIT2D_CRASH_SHA256)


def append_split_curve2d_requests(requests: list[str]) -> None:
    """Append exact comparisons; replay remaining inputs with verify_split_curve2d.py."""
    exact, _ = split2d_partition()
    requests.extend(exact)
