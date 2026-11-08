use super::*;

#[test]
fn audio_tracks_copy_metadata_from_a_borrowed_mpv_tree() {
    let tracks = {
        let mut keys =
            [c"type", c"id", c"title", c"lang", c"selected"].map(|key| key.as_ptr().cast_mut());
        let mut values = [
            MpvNode {
                value: MpvNodeValue {
                    string: c"audio".as_ptr().cast_mut(),
                },
                format: MPV_FORMAT_STRING,
            },
            MpvNode {
                value: MpvNodeValue { int64: 7 },
                format: MPV_FORMAT_INT64,
            },
            MpvNode {
                value: MpvNodeValue {
                    string: c" Deutsch ".as_ptr().cast_mut(),
                },
                format: MPV_FORMAT_STRING,
            },
            MpvNode {
                value: MpvNodeValue {
                    string: c"deu".as_ptr().cast_mut(),
                },
                format: MPV_FORMAT_STRING,
            },
            MpvNode {
                value: MpvNodeValue { flag: 1 },
                format: MPV_FORMAT_FLAG,
            },
        ];
        let mut map = MpvNodeList {
            num: values.len() as c_int,
            values: values.as_mut_ptr(),
            keys: keys.as_mut_ptr(),
        };
        let mut track = MpvNode {
            value: MpvNodeValue { list: &mut map },
            format: MPV_FORMAT_NODE_MAP,
        };
        let mut list = MpvNodeList {
            num: 1,
            values: &mut track,
            keys: ptr::null_mut(),
        };
        let root = MpvNode {
            value: MpvNodeValue { list: &mut list },
            format: MPV_FORMAT_NODE_ARRAY,
        };
        // SAFETY: All pointers refer to live arrays or static C strings until parsing returns.
        unsafe { audio_tracks_from_node(&root) }
    };

    // The returned tracks own their data after the source tree has gone out of scope.
    assert_eq!(
        tracks,
        [AudioTrack {
            id: 7,
            label: "Deutsch (deu)".to_string(),
            selected: true,
        }]
    );
}

#[test]
fn node_readers_reject_missing_data_and_incompatible_formats() {
    let empty = MpvNode {
        value: MpvNodeValue {
            list: ptr::null_mut(),
        },
        format: 0,
    };
    // SAFETY: No payload is initialized for format NONE, so readers must reject it.
    unsafe {
        assert!(node_int64(&empty).is_none());
        assert!(node_flag(Some(&empty)).is_none());
        assert!(node_flag(None).is_none());
        assert!(node_string(&empty).is_none());
        assert!(audio_tracks_from_node(&empty).is_empty());
    }

    let null_list = MpvNode {
        format: MPV_FORMAT_NODE_ARRAY,
        ..empty
    };
    // SAFETY: A null list pointer is handled as missing data without dereferencing it.
    assert!(unsafe { audio_tracks_from_node(&null_list) }.is_empty());
}
