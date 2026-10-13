//! Requested samples round upward, then select an actual attachment-supported count.
use ash::vk;

pub(super) fn select_attachment_samples(
    requested: i32,
    supported: vk::SampleCountFlags,
) -> vk::SampleCountFlags {
    if requested <= 0 {
        return vk::SampleCountFlags::TYPE_1;
    }
    let maximum = sample_counts()
        .into_iter()
        .rfind(|count| supported.contains(*count))
        .unwrap_or(vk::SampleCountFlags::TYPE_1)
        .as_raw();
    let clamped = (requested as u32).min(maximum).max(2);
    let target = clamped.next_power_of_two();
    sample_counts()
        .into_iter()
        .find(|count| count.as_raw() >= target && supported.contains(*count))
        .unwrap_or(vk::SampleCountFlags::from_raw(maximum))
}

fn sample_counts() -> [vk::SampleCountFlags; 7] {
    [
        vk::SampleCountFlags::TYPE_1,
        vk::SampleCountFlags::TYPE_2,
        vk::SampleCountFlags::TYPE_4,
        vk::SampleCountFlags::TYPE_8,
        vk::SampleCountFlags::TYPE_16,
        vk::SampleCountFlags::TYPE_32,
        vk::SampleCountFlags::TYPE_64,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requested_values_round_up_without_fabricating_unsupported_counts() {
        let host = vk::SampleCountFlags::TYPE_1
            | vk::SampleCountFlags::TYPE_2
            | vk::SampleCountFlags::TYPE_4
            | vk::SampleCountFlags::TYPE_8;
        for (request, actual) in [
            (0, 1),
            (1, 2),
            (2, 2),
            (3, 4),
            (4, 4),
            (6, 8),
            (8, 8),
            (99, 8),
        ] {
            assert_eq!(select_attachment_samples(request, host).as_raw(), actual);
        }
        let software = host & !vk::SampleCountFlags::TYPE_2;
        assert_eq!(
            select_attachment_samples(1, software),
            vk::SampleCountFlags::TYPE_4
        );
        assert_eq!(
            select_attachment_samples(2, software),
            vk::SampleCountFlags::TYPE_4
        );
    }
    #[test]
    fn no_multisample_support_and_nonpositive_requests_stay_single_sample() {
        assert_eq!(sample_counts().len(), 7);
        for request in [-2, 0, 1, 4, i32::MAX] {
            assert_eq!(
                select_attachment_samples(request, vk::SampleCountFlags::TYPE_1),
                vk::SampleCountFlags::TYPE_1
            );
        }
    }
}
