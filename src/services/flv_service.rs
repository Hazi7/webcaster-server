use axum::body::Bytes;

const FLV_HEADER: [u8; 13] = [
    0x46, 0x4C, 0x56, // 'FLV'
    0x01, // Version
    0x05, // Flags (audio & video)
    0x00, 0x00, 0x00, 0x09, // Header size
    0x00, 0x00, 0x00, 0x00, // PreviousTagSize0
];

/// FLV 服务结构体
pub struct FlvService {}

impl FlvService {
    /// 检查是否是关键帧
    pub fn is_key_frame(data: &[u8]) -> bool {
        data.len() > 11 && data[11] == 0x17 && data[12] == 0x01
    }

    /// 检查是否是序列头
    pub fn is_sequence_header(data: &[u8]) -> bool {
        data.len() > 11 && data.len() < 2000 && data[11] == 0x17 && data[12] == 0x00
    }

    pub fn is_aac_sequence(data: &[u8]) -> bool {
        data.len() > 11
            && data.len() < 2000
            && data[0] == 0x08
            && data[11] == 0xaf
            && data[12] == 0x00
    }

    /// 检查是否是元数据
    pub fn is_metadata(data: &[u8]) -> bool {
        data[0] == 0x12
    }

    /// 创建 FLV 头
    pub fn create_flv_header() -> Bytes {
        Bytes::from(FLV_HEADER.to_vec())
    }

    /// 处理视频时间戳

    pub fn process_video_timestamp(
        mut data: Vec<u8>,
        base_timestamp: &Option<u32>,
        first_timestamp: &mut Option<u32>,
    ) -> Vec<u8> {
        if data.len() < 6 {
            return data;
        }

        let timestamp = (data[4] as u32) << 16 | (data[5] as u32) << 8 | (data[6] as u32);

        if let Some(first) = first_timestamp {
            if timestamp == 0 {
                return data;
            }

            let new_timestamp = timestamp
                .wrapping_sub(*first)
                .wrapping_add(base_timestamp.unwrap());

            data[6] = (new_timestamp & 0xff) as u8;
            data[5] = ((new_timestamp >> 8) & 0xff) as u8;
            data[4] = ((new_timestamp >> 16) & 0xff) as u8;

            data
        } else {
            *first_timestamp = Some(timestamp);
            data[6] = (0 & 0xff) as u8;
            data[5] = ((0 >> 8) & 0xff) as u8;
            data[4] = ((0 >> 16) & 0xff) as u8;
            data
        }
    }
}
