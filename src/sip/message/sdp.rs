use std::str::FromStr;

use sdp_rs;

use vec1::vec1;

pub enum SdpSessionType {
    Play,
    Playback,
    Download,
    Talk,
}

impl ToString for SdpSessionType {
    fn to_string(&self) -> String {
        match &self {
            &Self::Play => String::from("Play"),
            &Self::Playback => String::from("Playback"),
            &Self::Download => String::from("Download"),
            &Self::Talk => String::from("Talk"),
        }
    }
}

impl FromStr for SdpSessionType {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Play" => Ok(Self::Play),
            "Playback" => Ok(Self::Playback),
            "Download" => Ok(Self::Download),
            "Talk" => Ok(Self::Talk),
            _ => Err(String::from(
                "Support lists: Play, Playback, Download, Talk",
            )),
        }
    }

    type Err = String;
}

pub fn generate_media_sdp(
    media_server_ip: &String,
    media_server_port: u16,
    device_gb_code: &String,
    setup_type: &String,
    session_type: SdpSessionType,
    start_ts: u64,
    stop_ts: u64,
) -> String {
    let mut attributes = vec![
        sdp_rs::lines::Attribute::Rtpmap(sdp_rs::lines::attribute::Rtpmap {
            payload_type: 96,
            encoding_name: String::from("PS"),
            clock_rate: 90000,
            encoding_params: None,
        }),
        sdp_rs::lines::Attribute::Rtpmap(sdp_rs::lines::attribute::Rtpmap {
            payload_type: 97,
            encoding_name: String::from("MPEG4"),
            clock_rate: 90000,
            encoding_params: None,
        }),
        sdp_rs::lines::Attribute::Rtpmap(sdp_rs::lines::attribute::Rtpmap {
            payload_type: 98,
            encoding_name: String::from("H264"),
            clock_rate: 90000,
            encoding_params: None,
        }),
        sdp_rs::lines::Attribute::Rtpmap(sdp_rs::lines::attribute::Rtpmap {
            payload_type: 99,
            encoding_name: String::from("HEVC"),
            clock_rate: 90000,
            encoding_params: None,
        }),
        sdp_rs::lines::Attribute::Recvonly {},
        sdp_rs::lines::Attribute::Other {
            0: String::from("streamMode"),
            1: Some(String::from("MAIN")),
        },
    ];
    if !setup_type.is_empty() {
        attributes.push(sdp_rs::lines::Attribute::Other {
            0: String::from("setup"),
            1: Some(setup_type.clone()),
        });
        attributes.push(sdp_rs::lines::Attribute::Other {
            0: String::from("connection"),
            1: Some(String::from("new")),
        });
    }

    // media description
    let media_desc = sdp_rs::MediaDescription {
        media: sdp_rs::lines::Media {
            media: sdp_rs::lines::media::MediaType::Video,
            port: media_server_port,
            num_of_ports: None,
            proto: if setup_type.is_empty() {
                sdp_rs::lines::media::ProtoType::RtpAvp
            } else {
                sdp_rs::lines::media::ProtoType::Other(String::from("TCP/RTP/AVP"))
            },
            fmt: String::from("96 97 98 99"),
        },
        attributes: attributes,
        info: None,
        connections: vec![sdp_rs::lines::Connection {
            nettype: sdp_rs::lines::common::Nettype::In,
            addrtype: sdp_rs::lines::common::Addrtype::Ip4,
            connection_address: sdp_rs::lines::connection::ConnectionAddress {
                base: std::net::IpAddr::V4(std::net::Ipv4Addr::from_str(media_server_ip).unwrap()),
                ttl: None,
                numaddr: None,
            },
        }],
        bandwidths: vec![],
        key: None,
    };

    let session_desc = sdp_rs::SessionDescription {
        version: sdp_rs::lines::Version::V0,
        origin: sdp_rs::lines::Origin {
            username: device_gb_code.clone(),
            sess_id: String::from("0"),
            sess_version: String::from("0"),
            nettype: sdp_rs::lines::common::Nettype::In,
            addrtype: sdp_rs::lines::common::Addrtype::Ip4,
            unicast_address: std::net::IpAddr::V4(
                std::net::Ipv4Addr::from_str(media_server_ip).unwrap(),
            ),
        },
        session_name: sdp_rs::lines::SessionName::new(session_type.to_string()),
        session_info: None,
        uri: None,
        times: vec1![sdp_rs::Time {
            active: sdp_rs::lines::Active {
                start: start_ts,
                stop: stop_ts
            },
            repeat: vec![],
            zone: None,
        }],
        media_descriptions: vec![media_desc],
        attributes: vec![],
        emails: vec![],
        phones: vec![],
        connection: None,
        bandwidths: vec![],
        key: None,
    };

    session_desc.to_string()
}
