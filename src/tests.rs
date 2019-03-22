/*  Copyright 2019 the Conwayste Developers.
 *
 *  This file is part of netwayste.
 *
 *  netwayste is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  netwayste is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with netwayste.  If not, see <http://www.gnu.org/licenses/>. */

mod net_tests {
    use std::{thread, time::{Instant, Duration}};
    use std::net::SocketAddr;
    use crate::net::*;
    use crate::futures::sync::mpsc;

    fn fake_socket_addr() -> SocketAddr {
        use std::net::{IpAddr, Ipv4Addr};
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)), 5678)
    }


    // `discord_older_packets()` tests are disabled  until after the necessity of the function is re-evaluated
    #[test]
    #[ignore]
    fn test_discard_older_packets_empty_queue() {
        let mut nm = NetworkManager::new();

        nm.tx_packets.discard_older_items();
        nm.rx_packets.discard_older_items();
        assert_eq!(nm.tx_packets.len(), 0);
        assert_eq!(nm.rx_packets.len(), 0);
    }

    #[test]
    #[ignore]
    fn test_discard_older_packets_under_limit_keeps_all_messages() {
        let mut nm = NetworkManager::new();
        let pkt = Packet::Request {
            sequence: 0,
            response_ack: None,
            cookie: None,
            action: RequestAction::None
        };

        nm.tx_packets.push_back(pkt.clone());
        nm.tx_packets.push_back(pkt.clone());
        nm.tx_packets.push_back(pkt.clone());

        nm.tx_packets.discard_older_items();
        assert_eq!(nm.tx_packets.len(), 3);

        nm.rx_packets.push_back(pkt.clone());
        nm.rx_packets.push_back(pkt.clone());
        nm.rx_packets.push_back(pkt.clone());

        nm.rx_packets.discard_older_items();
        assert_eq!(nm.rx_packets.len(), 3);
    }

    #[test]
    #[ignore]
    fn test_discard_older_packets_equal_to_limit() {
        let mut nm = NetworkManager::new();
        let pkt = Packet::Request {
            sequence: 0,
            response_ack: None,
            cookie: None,
            action: RequestAction::None
        };

        for _ in 0..NETWORK_QUEUE_LENGTH {
            nm.tx_packets.push_back(pkt.clone());
        }
        assert_eq!(nm.tx_packets.len(), NETWORK_QUEUE_LENGTH);
        nm.tx_packets.discard_older_items();
        assert_eq!(nm.tx_packets.len(), NETWORK_QUEUE_LENGTH-1);

        for _ in 0..NETWORK_QUEUE_LENGTH {
            nm.rx_packets.push_back(pkt.clone());
        }
        assert_eq!(nm.rx_packets.len(), NETWORK_QUEUE_LENGTH);
        nm.rx_packets.discard_older_items();
        assert_eq!(nm.rx_packets.len(), NETWORK_QUEUE_LENGTH);
    }

    #[test]
    #[ignore]
    fn test_discard_older_packets_exceeds_limit_retains_max() {
        let mut nm = NetworkManager::new();
        let pkt = Packet::Request {
            sequence: 0,
            response_ack: None,
            cookie: None,
            action: RequestAction::None
        };

        for _ in 0..NETWORK_QUEUE_LENGTH+10 {
            nm.tx_packets.push_back(pkt.clone());
        }
        assert_eq!(nm.tx_packets.len(), NETWORK_QUEUE_LENGTH+10);
        nm.tx_packets.discard_older_items();
        assert_eq!(nm.tx_packets.len(), NETWORK_QUEUE_LENGTH-1);

        for _ in 0..NETWORK_QUEUE_LENGTH+5 {
            nm.rx_packets.push_back(pkt.clone());
        }
        assert_eq!(nm.rx_packets.len(), NETWORK_QUEUE_LENGTH+5);
        nm.rx_packets.discard_older_items();
        assert_eq!(nm.rx_packets.len(), NETWORK_QUEUE_LENGTH);
    }

    #[test]
    fn test_buffer_item_queue_is_empty() {
        let mut nm = NetworkManager::new();
        let pkt = Packet::Request {
            sequence: 0,
            response_ack: None,
            cookie: None,
            action: RequestAction::None
        };

        nm.tx_packets.buffer_item(pkt);
        assert_eq!(nm.tx_packets.len(), 1);
    }

    #[test]
    fn test_buffer_item_sequence_number_reused() {
        let mut nm = NetworkManager::new();
        let pkt = Packet::Request {
            sequence: 0,
            response_ack: None,
            cookie: None,
            action: RequestAction::None
        };

        nm.tx_packets.buffer_item(pkt);
        let pkt = Packet::Request {
            sequence: 0,
            response_ack: None,
            cookie: None,
            action: RequestAction::LeaveRoom
        };

        nm.tx_packets.buffer_item(pkt);
        let pkt = nm.tx_packets.queue.back().unwrap();
        if let Packet::Request { sequence: _, response_ack: _, cookie: _, action } = pkt {
            assert_eq!(*action, RequestAction::None);
        }
    }

    #[test]
    fn test_buffer_item_basic_sequencing() {
        let mut nm = NetworkManager::new();
        let pkt = Packet::Request {
            sequence: 0,
            response_ack: None,
            cookie: None,
            action: RequestAction::None
        };

        nm.tx_packets.buffer_item(pkt);
        let pkt = Packet::Request {
            sequence: 1,
            response_ack: None,
            cookie: None,
            action: RequestAction::LeaveRoom
        };
        nm.tx_packets.buffer_item(pkt);
        assert_eq!(nm.tx_packets.len(), 2);
    }

    #[test]
    fn test_buffer_item_newer_packet_has_smaller_sequence_number() {
        let mut nm = NetworkManager::new();
        let pkt = Packet::Request {
            sequence: 1,
            response_ack: None,
            cookie: None,
            action: RequestAction::None
        };

        nm.tx_packets.buffer_item(pkt);
        let pkt = Packet::Request {
            sequence: 0,
            response_ack: None,
            cookie: None,
            action: RequestAction::LeaveRoom
        };
        nm.tx_packets.buffer_item(pkt);
        assert_eq!(nm.tx_packets.len(), 2);

        let pkt = nm.tx_packets.queue.back().unwrap();
        if let Packet::Request { sequence, response_ack: _, cookie: _, action:_ } = pkt {
            assert_eq!(*sequence, 1);
        }
    }


    // `buffer_item()` test with an enforced hard limit size is disabled until performance is re-examined
    #[test]
    #[ignore]
    fn test_buffer_item_max_queue_limit_maintained() {
        let mut nm = NetworkManager::new();
        for index in 0..NETWORK_QUEUE_LENGTH+5 {
            let pkt = Packet::Request {
                sequence: index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.tx_packets.buffer_item(pkt);
        }

        let mut iter =  nm.tx_packets.queue.iter();
        for index in 5..NETWORK_QUEUE_LENGTH+5 {
            let pkt = iter.next().unwrap();
            if let Packet::Request { sequence, response_ack: _, cookie: _, action:_ } = pkt {
                assert_eq!(*sequence, index as u64);
            }
        }
    }

    #[test]
    fn test_buffer_item_basic_contiguous_ascending() {
        let mut nm = NetworkManager::new();
        for index in 0..5 {
            let pkt = Packet::Request {
                sequence: index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        for index in 0..5 {
            let pkt = iter.next().unwrap();
            assert_eq!(index, pkt.sequence_number() as usize);
        }
    }

    #[test]
    fn test_buffer_item_basic_contiguous_descending() {
        let mut nm = NetworkManager::new();
        for index in (0..5).rev() {
            let pkt = Packet::Request {
                sequence: index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        for index in 0..5 {
            let pkt = iter.next().unwrap();
            assert_eq!(index, pkt.sequence_number() as usize);
        }
    }

    #[test]
    fn test_buffer_item_basic_sequential_gap_ascending() {
        let mut nm = NetworkManager::new();
        // TODO Replace with (x,y).step_by(z) once stable
        for index in [0,2,4,6,8,10].iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        for &index in [0,2,4,6,8,10].iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(index, pkt.sequence_number() as usize);
        }
    }

    #[test]
    fn test_buffer_item_basic_sequential_gap_descending() {
        let mut nm = NetworkManager::new();
        for index in [0,2,4,6,8,10].iter().rev() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        for index in [0,2,4,6,8,10].iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number());
        }
    }

    #[test]
    fn test_buffer_item_basic_random() {
        let mut nm = NetworkManager::new();
        for index in [5, 2, 9, 1, 0, 8, 6].iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        for index in [0, 1, 2, 5, 6, 8, 9].iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number() as usize);
        }
    }

    #[test]
    fn test_buffer_item_butterfly_pattern() {
        let mut nm = NetworkManager::new();
        // This one is fun because it tests the internal edges of (front_slice and back_slice)
        for index in [0, 10, 1, 9, 2, 8, 3, 7, 4, 6, 5].iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        for index in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10].iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number() as usize);
        }
    }

    #[test]
    fn test_buffer_item_basic_repetition() {
        let mut nm = NetworkManager::new();
        for index in [0, 0, 0, 0, 1, 2, 2, 2, 5].iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        for index in [0, 1, 2, 5].iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number() as usize);
        }
    }

    #[test]
    fn test_buffer_item_advanced_sequential_then_pseudorandom_then_sequential() {
        let mut nm = NetworkManager::new();

        for index in 0..5 {
            let pkt = Packet::Request {
                sequence: index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        for index in [10, 7, 11, 9, 12, 8, 99, 6].iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        for index in 13..20 {
            let pkt = Packet::Request {
                sequence: index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        let mut range = (0..20).collect::<Vec<usize>>();
        range.extend([99].iter().cloned()); // Add in 99
        range.remove(5); // But remove 5 since it was never included
        for index in range.iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number() as usize);
        }
    }

    #[test]
    fn test_buffer_item_advanced_reverse_sequential_then_random_then_reverse_sequential() {
        let mut nm = NetworkManager::new();

        for index in (0..5).rev() {
            let pkt = Packet::Request {
                sequence: index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        for index in [10, 7, 11, 9, 12, 8, 99, 6].iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        for index in (13..20).rev() {
            let pkt = Packet::Request {
                sequence: index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        let mut range = (0..20).collect::<Vec<usize>>();
        range.extend([99].iter().cloned()); // Add in 99
        range.remove(5); // But remove 5 since it was never included
        for index in range.iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number() as usize);
        }
    }

    #[test]
    fn test_buffer_item_basic_wrapping_case() {
        let mut nm = NetworkManager::new();
        let u64_max = <u64>::max_value();
        let start = u64_max - 5;

        for index in start..(start+5) {
            let pkt = Packet::Request {
                sequence: index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        {
            let pkt = Packet::Request {
                sequence: u64_max,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        for index in 0..5 {
            let pkt = Packet::Request {
                sequence: index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        let mut range = (start..u64_max).collect::<Vec<u64>>();
        range.extend([u64_max, 0, 1, 2, 3, 4].iter().cloned()); // Add in u64 max value plus others
        for index in range.iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number());
        }
    }

    #[test]
    fn test_buffer_item_basic_wrapping_case_then_out_of_order() {
        let mut nm = NetworkManager::new();
        let u64_max = <u64>::max_value();
        let start = u64_max - 5;

        for index in start..(start+5) {
            let pkt = Packet::Request {
                sequence: index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        {
            let pkt = Packet::Request {
                sequence: u64_max,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        for index in [5, 0, 4, 1, 3, 2].iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        let mut range = (start..u64_max).collect::<Vec<u64>>();
        range.extend([u64_max, 0, 1, 2, 3, 4, 5].iter().cloned()); // Add in u64 max value plus others
        for index in range.iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number());
        }
    }

    #[test]
    fn test_buffer_item_advanced_wrapping_case_everything_out_of_order() {
        let mut nm = NetworkManager::new();
        let u64_max = <u64>::max_value();
        let max_minus_5 = u64_max - 5;
        let max_minus_4 = u64_max - 4;
        let max_minus_3 = u64_max - 3;
        let max_minus_2 = u64_max - 2;
        let max_minus_1 = u64_max - 1;
        let zero = 0;
        let one = 1;
        let two = 2;
        let three = 3;

        let input_order = [ max_minus_4,
                            two,
                            max_minus_1,
                            max_minus_5,
                            u64_max,
                            three,
                            max_minus_2,
                            zero,
                            max_minus_3,
                            one ];

        for index in input_order.iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        let mut range = vec![];
        range.extend([max_minus_5, max_minus_4, max_minus_3, max_minus_2, max_minus_1, u64_max, zero, one, two, three]
                .iter()
                .cloned()); // Add in u64 max value plus others

        for index in range.iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number());
        }
    }

    #[test]
    fn test_buffer_item_advanced_max_sequence_number_arrives_after_a_wrap() {
        let mut nm = NetworkManager::new();
        let u64_max = <u64>::max_value();
        let max_minus_2 = u64_max - 2;
        let max_minus_1 = u64_max - 1;
        let two = 2;
        let three = 3;

        let input_order = [max_minus_1, max_minus_2, three, u64_max, two];

        for index in input_order.iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        let mut range = vec![];
        range.extend([max_minus_2, max_minus_1, u64_max, two, three].iter().cloned()); // Add in u64 max value plus others
        for index in range.iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number());
        }
    }

    #[test]
    fn test_buffer_item_advanced_oldest_sequence_number_arrived_last() {
        let mut nm = NetworkManager::new();
        let u64_max = <u64>::max_value();
        let max_minus_3 = u64_max - 3;
        let max_minus_2 = u64_max - 2;
        let max_minus_1 = u64_max - 1;
        let zero = 0;
        let one = 1;
        let two = 2;
        let three = 3;

        let input_order = [max_minus_1, max_minus_2, three, u64_max, two, one, zero, max_minus_3];

        for index in input_order.iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        let mut range = vec![];
        range.extend([max_minus_3, max_minus_2, max_minus_1, u64_max, zero, one, two, three].iter().cloned());

        for index in range.iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number());
        }
    }

    #[test]
    fn test_buffer_item_advanced_wrap_occurs_with_two_item_queue() {
        let mut nm = NetworkManager::new();
        let u64_max = <u64>::max_value();
        let max_minus_3 = u64_max - 3;
        let max_minus_2 = u64_max - 2;
        let max_minus_1 = u64_max - 1;
        let zero = 0;
        let one = 1;
        let two = 2;
        let three = 3;

        // Forward wrap occurs non-contiguously (aka [254, 0, ...] for bytes)
        let input_order = [max_minus_1, zero, three, u64_max, max_minus_2, one, two, max_minus_3];

        for index in input_order.iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        let mut range = vec![];
        range.extend([max_minus_3, max_minus_2, max_minus_1, u64_max, zero, one, two, three].iter().cloned());

        for index in range.iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number());
        }
    }

    #[test]
    fn test_buffer_item_advanced_wrap_occurs_with_two_item_queue_in_reverse() {
        let mut nm = NetworkManager::new();
        let u64_max = <u64>::max_value();
        let max_minus_3 = u64_max - 3;
        let max_minus_2 = u64_max - 2;
        let max_minus_1 = u64_max - 1;
        let zero = 0;
        let one = 1;
        let two = 2;
        let three = 3;

        // Wrap takes place in reverse order ( aka [0, 254, ...] for bytes)
        let input_order = [zero, max_minus_1, three, u64_max, max_minus_2, one, two, max_minus_3];

        for index in input_order.iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        let mut range = vec![];
        range.extend([max_minus_3, max_minus_2, max_minus_1, u64_max, zero, one, two, three].iter().cloned());

        for index in range.iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number());
        }
    }

    #[test]
    fn test_buffer_item_advanced_wrapping_case_max_arrives_first() {
        let mut nm = NetworkManager::new();
        let u64_max = <u64>::max_value();
        let max_minus_5 = u64_max - 5;
        let max_minus_4 = u64_max - 4;
        let max_minus_3 = u64_max - 3;
        let max_minus_2 = u64_max - 2;
        let max_minus_1 = u64_max - 1;
        let zero = 0;
        let one = 1;
        let two = 2;
        let three = 3;

        let input_order = [u64_max, max_minus_4, two, max_minus_1, max_minus_5, three, max_minus_2, zero, max_minus_3, one];

        for index in input_order.iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        let mut range = vec![];
        range.extend([max_minus_5, max_minus_4, max_minus_3, max_minus_2, max_minus_1, u64_max, zero, one, two, three].iter().cloned());

        for index in range.iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number());
        }
    }

    #[test]
    fn test_buffer_item_advanced_wrapping_case_sequence_number_descending() {
        let mut nm = NetworkManager::new();
        let u64_max = <u64>::max_value();
        let max_minus_5 = u64_max - 5;
        let max_minus_4 = u64_max - 4;
        let max_minus_3 = u64_max - 3;
        let max_minus_2 = u64_max - 2;
        let max_minus_1 = u64_max - 1;
        let zero = 0;
        let one = 1;
        let two = 2;
        let three = 3;

        let input_order = [three, two, one, zero, u64_max, max_minus_1, max_minus_2, max_minus_3, max_minus_4, max_minus_5];

        for index in input_order.iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        let mut range = vec![];
        range.extend([max_minus_5, max_minus_4, max_minus_3, max_minus_2, max_minus_1, u64_max, zero, one, two, three]
                .iter()
                .cloned());

        for index in range.iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number());
        }
    }

    #[test]
    fn test_buffer_item_advanced_wrapping_case_sequence_number_alternating() {
        let mut nm = NetworkManager::new();
        let u64_max = <u64>::max_value();
        let max_minus_5 = u64_max - 5;
        let max_minus_4 = u64_max - 4;
        let max_minus_3 = u64_max - 3;
        let max_minus_2 = u64_max - 2;
        let max_minus_1 = u64_max - 1;
        let zero = 0;
        let one = 1;
        let two = 2;
        let three = 3;

        let input_order = [max_minus_5, three, max_minus_4, two, max_minus_3, one, max_minus_2, zero, max_minus_1, u64_max];

        for index in input_order.iter() {
            let pkt = Packet::Request {
                sequence: *index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let mut iter = nm.rx_packets.queue.iter();
        let mut range = vec![];
        range.extend([max_minus_5, max_minus_4, max_minus_3, max_minus_2, max_minus_1, u64_max, zero, one, two, three]
                .iter()
                .cloned()); // Add in u64 max value plus others

        for index in range.iter() {
            let pkt = iter.next().unwrap();
            assert_eq!(*index, pkt.sequence_number());
        }
    }

    #[test]
    fn test_reinitialize_all_queues_cleared() {
        let mut nm = NetworkManager::new();
        let pkt = Packet::Request {
            sequence: 0,
            response_ack: None,
            cookie: None,
            action: RequestAction::None
        };

        for _ in 0..NETWORK_QUEUE_LENGTH {
            nm.tx_packets.push_back(pkt.clone());
        }
        assert_eq!(nm.tx_packets.len(), NETWORK_QUEUE_LENGTH);

        let _chat_msg = BroadcastChatMessage::new(0, "chatchat".to_owned(), "chatchat".to_owned());
    }

    #[test]
    fn test_get_contiguous_packets_count() {
        let mut nm = NetworkManager::new();
        for index in 0..5 {
            let pkt = Packet::Request {
                sequence: index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }
        for index in 8..10 {
            let pkt = Packet::Request {
                sequence: index as u64,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };
            nm.rx_packets.buffer_item(pkt);
        }

        let count = nm.rx_packets.get_contiguous_packets_count(0);
        assert_eq!(count, 5);
        let mut iter = nm.rx_packets.as_queue_type().iter();
        for index in 0..5 {
            let pkt = iter.next().unwrap();
            assert_eq!(index, pkt.sequence_number() as usize);
            // Verify that the packet is not dequeued
            assert_eq!(index, nm.rx_packets.as_queue_type().get(index).unwrap().sequence_number() as usize);
        }
    }

    #[test]
    fn test_get_retransmit_indices() {
        let mut nm = NetworkManager::new();
        for i in 0..5 {
            let pkt = Packet::Request {
                sequence: i,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };

            nm.tx_packets.buffer_item(pkt.clone());

            if i < 3 {
                let attempt: &mut NetAttempt = nm.tx_packets.attempts.back_mut().unwrap();
                attempt.time = Instant::now() - Duration::from_secs(i+1);
            }
        }
        assert_eq!(nm.tx_packets.get_retransmit_indices().len(), 3);
        thread::sleep(Duration::from_millis(2000));
        assert_eq!(nm.tx_packets.get_retransmit_indices().len(), 5);
    }

    #[test]
    fn test_retransmit_expired_tx_packets_no_expirations() {
        let mut nm = NetworkManager::new();

        for i in 0..5 {
            let pkt = Packet::Request {
                sequence: i,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };

            nm.tx_packets.buffer_item(pkt.clone());
        }

        let indices = nm.tx_packets.get_retransmit_indices();

        let (udp_tx, _) = mpsc::unbounded();
        let addr = fake_socket_addr();
        nm.retransmit_expired_tx_packets(&udp_tx, addr, None, &indices);

        for i in 0..5 {
            assert_eq!(nm.tx_packets.attempts.get(i).unwrap().retries, 0);
        }
    }

    #[test]
    fn test_retransmit_expired_tx_packets_basic_retries() {
        let mut nm = NetworkManager::new();

        for i in 0..5 {
            let pkt = Packet::Request {
                sequence: i,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };

            nm.tx_packets.buffer_item(pkt.clone());

           if i < 3 {
                let attempt: &mut NetAttempt = nm.tx_packets.attempts.back_mut().unwrap();
                attempt.time = Instant::now() - Duration::from_secs(i+1);
            }
        }

        let indices = nm.tx_packets.get_retransmit_indices();

        let (udp_tx, _) = mpsc::unbounded();
        let addr = fake_socket_addr();
        nm.retransmit_expired_tx_packets(&udp_tx, addr, None, &indices);

        for i in 0..3 {
            assert_eq!(nm.tx_packets.attempts.get(i).unwrap().retries, 1);
        }
        for i in 3..5 {
            assert_eq!(nm.tx_packets.attempts.get(i).unwrap().retries, 0);
        }
    }

    #[test]
    fn test_retransmit_expired_tx_packets_aggressive_retries() {
        let mut nm = NetworkManager::new();

        for i in 0..5 {
            let pkt = Packet::Request {
                sequence: i,
                response_ack: None,
                cookie: None,
                action: RequestAction::None
            };

            nm.tx_packets.buffer_item(pkt.clone());

           if i < 3 {
                let attempt: &mut NetAttempt = nm.tx_packets.attempts.back_mut().unwrap();
                attempt.time = Instant::now() - Duration::from_secs(i+1);
            }
        }

        // After 2 attempts, aggressive mode should kick in
        for _ in 0..5 {
            let indices = nm.tx_packets.get_retransmit_indices();

            println!("{:?}", indices);

            let (udp_tx, _) = mpsc::unbounded();
            let addr = fake_socket_addr();
            nm.retransmit_expired_tx_packets(&udp_tx, addr, None, &indices);

            for j in 0..indices.len() {
                let attempt: &mut NetAttempt = nm.tx_packets.attempts.get_mut(j).unwrap();
                attempt.time = Instant::now() - Duration::from_secs( 1u64);
            }
        }

        for i in 0..3 {
            // 5 + 2 + 2 + 3
            assert_eq!(nm.tx_packets.attempts.get(i).unwrap().retries, 11);
        }
        for i in 3..5 {
            assert_eq!(nm.tx_packets.attempts.get(i).unwrap().retries, 0);
        }

    }

}

mod netwayste_client_tests {

}

mod netwayste_server_tests {

}