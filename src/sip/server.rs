use tokio::{self, io::AsyncReadExt};

use crate::sip::handler::SipHandler;

use crate::utils::{cli::CommandLines, kmp::Kmp};

pub static DOUBLE_CR_LF_STR: &str = "\r\n\r\n";
pub static DOUBLE_CR_LF_BYTES: &[u8; 4] = b"\r\n\r\n";
pub static SIP_BYTES: [u8; 3] = [b'S', b'I', b'P'];
pub static CONTENT_LENGTH_BYTES: &[u8; 15] = b"Content-Length:";

pub async fn bind(
    cli_args: &CommandLines,
) -> Result<(tokio::net::UdpSocket, tokio::net::TcpListener), std::io::Error> {
    let local_addr = format!(
        "{host}:{port}",
        host = cli_args.host,
        port = cli_args.sip_port
    );

    // udp server
    match tokio::net::UdpSocket::bind(&local_addr).await {
        Err(e) => {
            tracing::error!("UdpSocket::bind({}) error, e: {:?}", &local_addr, e);
            return Err(e);
        }
        Ok(udp_socket) => {
            tracing::info!("UdpSocket::bind({}) ok", &local_addr);

            // tcp server
            match tokio::net::TcpListener::bind(&local_addr).await {
                Err(e) => {
                    tracing::error!("TcpListener::bind({}) error, e: {:?}", &local_addr, e);
                    return Err(e);
                }
                Ok(tcp_listener) => {
                    tracing::info!("TcpListener::bind({}) ok", &local_addr);
                    return Ok((udp_socket, tcp_listener));
                }
            }
        }
    }
}

fn parse_sip_message(buffer: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    // kmp search content-length
    if let Some(mut content_length_begin_pos) =
        Kmp::find_first_occurrence(buffer, CONTENT_LENGTH_BYTES)
    {
        content_length_begin_pos += CONTENT_LENGTH_BYTES.len();

        let mut content_length_end_pos = content_length_begin_pos;
        while content_length_end_pos < buffer.len() {
            if buffer[content_length_end_pos] != b'\r' && buffer[content_length_end_pos] != b'\n' {
                content_length_end_pos += 1;
            } else {
                break;
            }
        }

        let content_length =
            String::from_utf8_lossy(&buffer[content_length_begin_pos..content_length_end_pos])
                .trim()
                .parse::<usize>()
                .unwrap_or(0);

        // kmp search \r\n\r\n
        if let Some(mut content_pos) =
            Kmp::find_first_occurrence(&buffer[content_length_end_pos..], DOUBLE_CR_LF_BYTES)
        {
            if buffer.len() - content_pos - DOUBLE_CR_LF_BYTES.len() >= content_length {
                content_pos += content_length_end_pos + DOUBLE_CR_LF_BYTES.len() + content_length;
                let sip_message = buffer[..content_pos].to_vec();
                let remaining = buffer[content_pos..].to_vec();
                return Some((sip_message, remaining));
            }
        }
    }

    None
}

pub async fn run_forever(
    cli_args: CommandLines,
    sip_handler: std::sync::Arc<SipHandler>,
) -> Result<(), std::io::Error> {
    // udp server
    let sip_handler_udp = sip_handler.clone();
    let udp_server_handle = tokio::spawn(async move {
        let mut recv_buff = Vec::<u8>::default();
        recv_buff.resize(cli_args.socket_recv_buffer_size, 0);

        loop {
            match sip_handler_udp
                .sip_udp_socket
                .recv_from(recv_buff.as_mut_slice())
                .await
            {
                Err(e) => {
                    tracing::error!("UdpSocket::recv_from error, e: {:?}", e);
                }
                Ok((amount, addr)) => {
                    sip_handler_udp
                        .dispatch(addr, None, &recv_buff.as_slice()[..amount])
                        .await;
                }
            }
        }
    });

    // tcp server
    let tcp_server_handle = tokio::spawn(async move {
        loop {
            match sip_handler.sip_tcp_listener.accept().await {
                Err(e) => {
                    tracing::error!("TcpListener::accept error: {:?}", e);
                }
                Ok((tcp_stream, addr)) => {
                    let sip_handler_cloned = sip_handler.clone();
                    tokio::spawn(async move {
                        let mut buffer = Vec::new();

                        let (mut tcp_stream_reader, tcp_stream_writer) = tcp_stream.into_split();

                        let tcp_stream_mutex_arc =
                            std::sync::Arc::new(tokio::sync::Mutex::new(tcp_stream_writer));

                        loop {
                            let mut recv_buff = vec![0; 1024];
                            let n = match tcp_stream_reader
                                .read(&mut recv_buff)
                                .await
                            {
                                Ok(0) => return, // connection closed
                                Ok(n) => n,
                                Err(e) => {
                                    tracing::error!("TcpStream::read error, e: {:?}", e);
                                    return;
                                }
                            };

                            buffer.extend_from_slice(&recv_buff[..n]);

                            while let Some((message, remaining)) = parse_sip_message(&buffer) {
                                sip_handler_cloned
                                    .dispatch(
                                        addr,
                                        Some(tcp_stream_mutex_arc.clone()),
                                        message.as_slice(),
                                    )
                                    .await;
                                buffer = remaining;
                            }
                        }
                    });
                }
            }
        }
    });

    let _ = tokio::join!(udp_server_handle, tcp_server_handle);

    return Ok(());
}
