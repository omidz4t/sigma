use crate::{
    Result,
    account::AccountLock,
    cli::Args,
    help::{COMMAND_HELP, command_help},
    shared::*,
};
use deltachat::{
    chat::{self, Chat, ChatId},
    contact::Contact,
    context::ContextBuilder,
    message::Viewtype,
};
// macOS TMPDIR can be too long for sockaddr_un, even with short filenames.
// Use a short, unique, private root for tests that exercise Unix sockets.
fn socket_test_directory() -> std::path::PathBuf {
    #[cfg(unix)]
    let base = std::path::PathBuf::from("/tmp");
    #[cfg(not(unix))]
    let base = std::env::temp_dir();
    let root = base.join(format!("sg-{}", uuid::Uuid::new_v4().simple()));
    let mut builder = std::fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(&root).unwrap();
    root
}

fn parse(args: &[&str]) -> Result<Args> {
    Args::parse(args.iter().map(|s| s.to_string()))
}
#[test]
fn options_anywhere_and_literal_message() {
    let args = parse(&[
        "--data-path",
        "/tmp/bot",
        "send-message",
        "42",
        "--",
        "--hello",
    ])
    .unwrap();
    args.validate().unwrap();
    assert_eq!(args.positional, ["42", "--hello"]);
    assert_eq!(args.option("--data-path"), Some("/tmp/bot"));
}
#[test]
fn securejoin_arguments() {
    for invite in [
        "https://i.delta.chat/#fingerprint&i=token&s=auth",
        "https://i.delta.chat#fingerprint&i=token&s=auth",
        "OPENPGP4FPR:ABCD#a=alice%40example.org&i=token&s=auth",
    ] {
        parse(&["create-contact", invite, "--timeout", "30"])
            .unwrap()
            .validate()
            .unwrap();
        parse(&["send-message", invite, "hello"])
            .unwrap()
            .validate()
            .unwrap();
    }
    for input in [
        vec!["create-contact", "42"],
        vec!["create-contact"],
        vec!["send-message", "42", "hi", "--timeout", "0"],
        vec!["status", "--timeout", "1"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err());
    }
}
#[tokio::test(flavor = "multi_thread")]
async fn file_attachment_uses_core_blob_storage() {
    let directory =
        std::env::temp_dir().join(format!("sigma-attachment-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let source = directory.join("test document.txt");
    std::fs::write(&source, b"attachment content").unwrap();
    let args = parse(&[
        "send-file",
        "42",
        source.to_str().unwrap(),
        "--caption",
        "Here it is",
    ])
    .unwrap();
    args.validate().unwrap();
    let context = ContextBuilder::new(directory.join("account.db"))
        .open()
        .await
        .unwrap();
    let message =
        attachment_message(&context, &source, args.option("--caption"), Viewtype::File).unwrap();
    assert_eq!(message.get_text(), "Here it is");
    let blob = message.get_file(&context).unwrap();
    assert_ne!(blob, source);
    assert_eq!(std::fs::read(blob).unwrap(), b"attachment content");
    assert_eq!(std::fs::read(&source).unwrap(), b"attachment content");
    for input in [
        vec!["send-file", "42"],
        vec!["send-file", "0", source.to_str().unwrap()],
        vec!["send-file", "42", "/no/such/sigma-file"],
        vec!["send-message", "42", "hi", "--caption", "caption"],
        vec!["init", "--caption", "caption"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err());
    }
    let photo = directory.join("photo.png");
    std::fs::write(
        &photo,
        include_bytes!(concat!(env!("OUT_DIR"), "/photo.png")),
    )
    .unwrap();
    for target in ["42", "https://i.delta.chat/#invite"] {
        parse(&[
            "send-photo",
            target,
            photo.to_str().unwrap(),
            "--caption",
            "Photo",
            "--timeout",
            "30",
        ])
        .unwrap()
        .validate()
        .unwrap();
    }
    let image = attachment_message(&context, &photo, Some("Photo"), Viewtype::Image).unwrap();
    assert_eq!(image.get_viewtype(), Viewtype::Image);
    assert_eq!(image.get_text(), "Photo");
    assert_eq!(
        std::fs::read(image.get_file(&context).unwrap()).unwrap(),
        include_bytes!(concat!(env!("OUT_DIR"), "/photo.png"))
    );
    assert!(
        parse(&["send-photo", "42", photo.to_str().unwrap(), "--as-text"])
            .unwrap()
            .validate()
            .is_err()
    );
    assert!(
        parse(&["send-photo", "42", "/no/such/photo.png"])
            .unwrap()
            .validate()
            .is_err()
    );
    assert!(parse(&["send-photo", "42"]).unwrap().validate().is_err());
    drop(image);
    drop(message);
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn text_file_preserves_contents_and_rejects_invalid_input() {
    let path =
        std::env::temp_dir().join(format!("sigma-text-file-test-{}.txt", std::process::id()));
    let text = "Hello 🌍\nSecond line\n";
    std::fs::write(&path, text).unwrap();
    for target in ["42", "https://i.delta.chat/#invite"] {
        for input in [
            vec!["send-file", target, path.to_str().unwrap(), "--as-text"],
            vec!["--as-text", "send-file", target, path.to_str().unwrap()],
        ] {
            parse(&input).unwrap().validate().unwrap();
        }
    }
    let message = text_file_message(&path).unwrap();
    assert_eq!(message.get_text(), text);
    assert_eq!(message.get_viewtype(), Viewtype::Text);
    for content in [b"".as_slice(), b" \n\t".as_slice(), &[0xff, 0xfe]] {
        std::fs::write(&path, content).unwrap();
        assert!(text_file_message(&path).is_err());
    }
    for input in [
        vec!["send-message", "42", "hi", "--as-text"],
        vec![
            "send-file",
            "42",
            path.to_str().unwrap(),
            "--as-text",
            "--caption",
            "hi",
        ],
    ] {
        assert!(parse(&input).unwrap().validate().is_err());
    }
    assert!(parse(&["send-file", "42", path.to_str().unwrap(), "--as-text=true"]).is_err());
    assert!(
        parse(&[
            "send-file",
            "42",
            path.to_str().unwrap(),
            "--as-text",
            "--as-text"
        ])
        .is_err()
    );
    std::fs::remove_file(&path).unwrap();
    assert!(text_file_message(&path).is_err());
}

#[test]
fn join_qr_export_and_options() {
    parse(&["join-link"]).unwrap().validate().unwrap();
    parse(&["give-qr", "--output", "invite.svg"])
        .unwrap()
        .validate()
        .unwrap();
    assert!(
        parse(&["status", "--output", "invite.svg"])
            .unwrap()
            .validate()
            .is_err()
    );
    let output = std::env::temp_dir().join(format!("sigma-qr-test-{}.svg", std::process::id()));
    save_join_qr("https://i.delta.chat/#test-invitation", &output).unwrap();
    let svg = std::fs::read_to_string(&output).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(save_join_qr("another link", &output).is_err());
    assert_eq!(std::fs::read_to_string(&output).unwrap(), svg);
    std::fs::remove_file(output).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn broadcast_channel_is_persistent_and_distinct_from_direct_chats() {
    let directory = std::env::temp_dir().join(format!("sigma-channel-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let database = directory.join("account.db");
    let context = ContextBuilder::new(database.clone()).open().await.unwrap();
    let channel = chat::create_broadcast(&context, "Updates".to_string())
        .await
        .unwrap();
    assert_eq!(
        outgoing_channel(&context, channel.to_u32()).await.unwrap(),
        channel
    );
    assert_eq!(
        Chat::load_from_db(&context, channel)
            .await
            .unwrap()
            .get_name(),
        "Updates"
    );
    let contact = Contact::create(&context, "Alice", "alice@example.org")
        .await
        .unwrap();
    let direct = ChatId::create_for_contact(&context, contact).await.unwrap();
    assert!(outgoing_channel(&context, direct.to_u32()).await.is_err());
    drop(context);
    let context = ContextBuilder::new(database).open().await.unwrap();
    assert!(outgoing_channel(&context, channel.to_u32()).await.is_ok());
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn channel_arguments() {
    for input in [
        vec!["create-channel", "Updates"],
        vec!["channel-link", "12"],
        vec!["invite-to-channel", "12", "10"],
        vec!["send-channel-message", "12", "Hello subscribers"],
    ] {
        parse(&input).unwrap().validate().unwrap();
    }
    for input in [
        vec!["create-channel", " "],
        vec!["channel-link", "0"],
        vec!["invite-to-channel", "12", "1"],
        vec!["invite-to-channel", "12", "invalid"],
        vec!["send-channel-message", "12", " "],
        vec!["send-channel-message", "12"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err());
    }
}

#[test]
fn receive_and_read_arguments() {
    for input in [
        vec!["receive"],
        vec!["receive", "--timeout", "30"],
        vec!["read-messages", "12", "--mark-read"],
    ] {
        parse(&input).unwrap().validate().unwrap();
    }
    for input in [
        vec!["receive", "--timeout", "0"],
        vec!["receive", "extra"],
        vec!["read-messages", "0"],
        vec!["messages", "12", "--mark-read"],
        vec!["receive", "--mark-read"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err());
    }
    assert!(parse(&["read-messages", "12", "--mark-read=true"]).is_err());
}

#[test]
fn reaction_arguments_and_normalization() {
    for input in [
        vec!["react", "12", "👍"],
        vec!["react", "12", "👩‍💻", "--timeout", "30"],
        vec!["react", "12"],
        vec!["react", "12", ""],
        vec!["reactions", "12"],
    ] {
        parse(&input).unwrap().validate().unwrap();
    }
    for input in [
        vec!["react"],
        vec!["react", "0", "👍"],
        vec!["react", "12", "👍 ❤️"],
        vec!["react", "12", " "],
        vec!["react", "12", "👍", "--timeout", "0"],
        vec!["reactions", "no-id"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err());
    }
    assert!(validate_reaction(&"x".repeat(30)).is_err());
}

#[test]
fn command_specific_help_and_literal_arguments() {
    for entry in COMMAND_HELP {
        assert!(!entry.summary.is_empty());
        assert!(!entry.description.is_empty());
        assert!(!entry.example.is_empty());
        assert!(crate::help::general_help().contains(entry.summary));
        for flag in ["--help", "-h"] {
            let args = parse(&[entry.name, flag]).unwrap();
            assert_eq!(args.command, flag);
            assert_eq!(args.positional, [entry.name]);
            let help = command_help(&args.positional[0]).unwrap();
            assert!(help.contains(&format!("USAGE: sigma {}", entry.name)));
            assert_eq!(
                help.contains("\n  --data-path "),
                !entry.name.starts_with("account-") && entry.name != "agent-act"
            );
            assert!(help.contains("--json"));
            assert!(help.contains("EXAMPLE:"));
        }
    }
    assert!(command_help("unknown").is_err());
    let args = parse(&["send-message", "10", "--", "--help"]).unwrap();
    assert_eq!(args.command, "send-message");
    assert_eq!(args.positional, ["10", "--help"]);
    let args = parse(&["--data-path", "/missing", "send-photo", "--help"]).unwrap();
    assert_eq!(args.positional, ["send-photo"]);
    assert!(!command_help("send-photo").unwrap().contains("--as-text"));
    assert!(!command_help("status").unwrap().contains("--timeout"));
    assert!(parse(&["--help"]).unwrap().positional.is_empty());
}

#[test]
fn account_lock_is_released_on_drop() {
    let path = std::env::temp_dir().join(format!("sigma-lock-test-{}", std::process::id()));
    let first = AccountLock::acquire(path.clone()).unwrap();
    assert!(AccountLock::acquire(path.clone()).is_err());
    drop(first);
    let second = AccountLock::acquire(path.clone()).unwrap();
    drop(second);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn invalid_input_is_rejected() {
    for input in [
        vec!["init", "--wat", "x"],
        vec!["init", "--name"],
        vec!["init", "--name", "a", "--name", "b"],
    ] {
        assert!(parse(&input).is_err());
    }
    for input in [
        vec!["send-message", "0", "hi"],
        vec!["messages", "oops"],
        vec!["status", "--name", "x"],
        vec!["send-message", "42"],
        vec!["unknown"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err());
    }
}

#[test]
fn reply_arguments() {
    parse(&["reply", "42", "Thanks!"])
        .unwrap()
        .validate()
        .unwrap();
    parse(&["reply", "42", "--", "--help"])
        .unwrap()
        .validate()
        .unwrap();
    for input in [
        vec!["reply"],
        vec!["reply", "42"],
        vec!["reply", "0", "hi"],
        vec!["reply", "no-id", "hi"],
        vec!["reply", "42", " "],
        vec!["reply", "42", "hi", "--caption", "caption"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn quoted_reply_preserves_parent_and_chat() {
    use deltachat::message::Message;
    let directory = std::env::temp_dir().join(format!("sigma-reply-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let context = ContextBuilder::new(directory.join("account.db"))
        .open()
        .await
        .unwrap();
    let mut original = Message::new_text("Original message".to_owned());
    let id = chat::add_device_msg(&context, None, Some(&mut original))
        .await
        .unwrap();
    let original = Message::load_from_db(&context, id).await.unwrap();
    let (chat_id, reply) = crate::handlers::reply::prepare(&context, id, "Reply 🌍\nsecond line")
        .await
        .unwrap();
    assert_eq!(chat_id, original.get_chat_id());
    assert_eq!(reply.get_text(), "Reply 🌍\nsecond line");
    assert_eq!(reply.quoted_text().unwrap(), "Original message");
    assert_eq!(reply.parent(&context).await.unwrap().unwrap().get_id(), id);
    assert!(
        crate::handlers::reply::prepare(&context, id, " ")
            .await
            .is_err()
    );
    assert!(
        crate::handlers::reply::prepare(&context, deltachat::message::MsgId::new(u32::MAX), "hi")
            .await
            .is_err()
    );
    drop(reply);
    drop(original);
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn default_account_directory_and_explicit_override() {
    use crate::account::data_path;
    use std::{ffi::OsStr, path::PathBuf};
    assert_eq!(
        data_path(None, Some(OsStr::new("/home/test user"))).unwrap(),
        PathBuf::from("/home/test user/.config/sigma")
    );
    assert_eq!(
        data_path(Some("./bot-data"), None).unwrap(),
        PathBuf::from("./bot-data")
    );
    assert_eq!(
        data_path(Some("/tmp/custom-bot"), Some(OsStr::new("/home/test"))).unwrap(),
        PathBuf::from("/tmp/custom-bot")
    );
    assert!(data_path(None, None).is_err());
    assert!(data_path(None, Some(OsStr::new(""))).is_err());
    assert!(data_path(None, Some(OsStr::new("relative-home"))).is_err());
}

#[test]
fn create_group_arguments() {
    parse(&["create-group", "Team chat"])
        .unwrap()
        .validate()
        .unwrap();
    for input in [
        vec!["create-group"],
        vec!["create-group", " "],
        vec!["create-group", "Team", "extra"],
        vec!["create-group", "Team", "--caption", "no"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn created_group_is_encrypted_and_persistent() {
    let directory = std::env::temp_dir().join(format!("sigma-group-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let database = directory.join("account.db");
    let context = ContextBuilder::new(database.clone()).open().await.unwrap();
    let id = chat::create_group(&context, "Team chat").await.unwrap();
    let group = Chat::load_from_db(&context, id).await.unwrap();
    assert_eq!(group.get_type(), deltachat::constants::Chattype::Group);
    assert_eq!(group.get_name(), "Team chat");
    assert!(group.is_encrypted(&context).await.unwrap());
    assert!(group.is_self_in_chat(&context).await.unwrap());
    drop(group);
    drop(context);
    let context = ContextBuilder::new(database).open().await.unwrap();
    assert_eq!(
        Chat::load_from_db(&context, id).await.unwrap().get_name(),
        "Team chat"
    );
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn add_to_group_arguments() {
    parse(&["add-to-group", "12", "10"])
        .unwrap()
        .validate()
        .unwrap();
    for input in [
        vec!["add-to-group"],
        vec!["add-to-group", "12"],
        vec!["add-to-group", "0", "10"],
        vec!["add-to-group", "bad", "10"],
        vec!["add-to-group", "12", "1"],
        vec!["add-to-group", "12", "bad"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn group_membership_uses_core_and_rejects_wrong_chats() {
    use crate::handlers::add_to_group::add_member;
    use deltachat::{
        contact::{self, ContactId},
        securejoin,
        transport::add_pseudo_transport,
    };
    let directory =
        std::env::temp_dir().join(format!("sigma-group-member-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let context = ContextBuilder::new(directory.join("bot.db"))
        .open()
        .await
        .unwrap();
    let person = ContextBuilder::new(directory.join("person.db"))
        .open()
        .await
        .unwrap();
    // Provision local test-only identities without connecting to any service.
    add_pseudo_transport(&person, "person@example.org")
        .await
        .unwrap();
    securejoin::get_securejoin_qr(&person, None).await.unwrap();
    let card = contact::make_vcard(&person, &[ContactId::SELF])
        .await
        .unwrap();
    let id = contact::import_vcard(&context, &card).await.unwrap()[0];
    let group = chat::create_group(&context, "Team").await.unwrap();
    add_member(&context, group, id).await.unwrap();
    assert!(chat::is_contact_in_chat(&context, group, id).await.unwrap());
    add_member(&context, group, id).await.unwrap();
    assert_eq!(
        chat::get_chat_contacts(&context, group)
            .await
            .unwrap()
            .iter()
            .filter(|entry| **entry == id)
            .count(),
        1
    );
    let address = Contact::create(&context, "Address only", "address@example.org")
        .await
        .unwrap();
    assert!(
        add_member(&context, group, address)
            .await
            .unwrap_err()
            .to_string()
            .contains("key contact")
    );
    let channel = chat::create_broadcast(&context, "Updates".to_owned())
        .await
        .unwrap();
    assert!(add_member(&context, channel, id).await.is_err());
    let direct = ChatId::create_for_contact(&context, id).await.unwrap();
    assert!(add_member(&context, direct, id).await.is_err());
    assert!(add_member(&context, group, ContactId::SELF).await.is_err());
    drop(person);
    drop(context);
    let context = ContextBuilder::new(directory.join("bot.db"))
        .open()
        .await
        .unwrap();
    assert!(chat::is_contact_in_chat(&context, group, id).await.unwrap());
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn webxdc_package_preparation_and_arguments() {
    use crate::handlers::send_webxdc::prepare;
    let directory = std::env::temp_dir().join(format!("sigma-webxdc-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let source = directory.join("app.xdc");
    std::fs::write(
        &source,
        include_bytes!(concat!(env!("OUT_DIR"), "/minimal.xdc")),
    )
    .unwrap();
    for input in [
        vec!["send-webxdc", "10", source.to_str().unwrap()],
        vec![
            "send-webxdc",
            "12",
            source.to_str().unwrap(),
            "--chat",
            "--caption",
            "App",
        ],
        vec![
            "send-webxdc",
            "https://i.delta.chat/#invite",
            source.to_str().unwrap(),
            "--timeout",
            "30",
        ],
    ] {
        parse(&input).unwrap().validate().unwrap();
    }
    for input in [
        vec!["send-webxdc", "0", source.to_str().unwrap()],
        vec![
            "send-webxdc",
            "https://i.delta.chat/#invite",
            source.to_str().unwrap(),
            "--chat",
        ],
        vec!["send-webxdc", "10", source.to_str().unwrap(), "--as-text"],
        vec!["status", "--chat"],
        vec!["send-webxdc", "10"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err());
    }
    let context = ContextBuilder::new(directory.join("account.db"))
        .open()
        .await
        .unwrap();
    let message = prepare(&context, &source, Some("App caption"))
        .await
        .unwrap();
    assert_eq!(message.get_viewtype(), Viewtype::Webxdc);
    assert_eq!(message.get_text(), "App caption");
    assert!(
        String::from_utf8(
            message
                .get_webxdc_blob(&context, "index.html")
                .await
                .unwrap()
        )
        .unwrap()
        .contains("Sigma test app")
    );
    assert_eq!(
        std::fs::read(&source).unwrap(),
        include_bytes!(concat!(env!("OUT_DIR"), "/minimal.xdc"))
    );
    let invalid = directory.join("invalid.xdc");
    std::fs::write(&invalid, b"not a zip archive").unwrap();
    assert!(prepare(&context, &invalid, None).await.is_err());
    let mut empty_zip = vec![0x50, 0x4b, 0x05, 0x06];
    empty_zip.extend([0u8; 18]);
    std::fs::write(&invalid, empty_zip).unwrap();
    assert!(prepare(&context, &invalid, None).await.is_err());
    assert!(
        prepare(&context, &directory.join("missing.xdc"), None)
            .await
            .is_err()
    );
    assert!(
        prepare(&context, &directory.join("app.txt"), None)
            .await
            .is_err()
    );
    drop(message);
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

async fn offline_identity(directory: &std::path::Path, name: &str) -> deltachat::context::Context {
    let context = ContextBuilder::new(directory.join(format!("{name}.db")))
        .open()
        .await
        .unwrap();
    deltachat::transport::add_pseudo_transport(&context, &format!("{name}@example.org"))
        .await
        .unwrap();
    deltachat::securejoin::get_securejoin_qr(&context, None)
        .await
        .unwrap();
    context
}

async fn offline_key_contact(
    context: &deltachat::context::Context,
    peer: &deltachat::context::Context,
) -> deltachat::contact::ContactId {
    let card = deltachat::contact::make_vcard(peer, &[deltachat::contact::ContactId::SELF])
        .await
        .unwrap();
    deltachat::contact::import_vcard(context, &card)
        .await
        .unwrap()[0]
}

#[test]
fn expanded_commands_validate_options_and_targets() {
    for input in [
        vec!["send-message", "12", "Hello", "--chat", "--json"],
        vec!["group-members", "12"],
        vec!["channel-members", "12"],
        vec!["rename-group", "12", "Team"],
        vec!["rename-channel", "12", "News"],
        vec!["remove-from-group", "12", "10"],
        vec!["remove-from-channel", "12", "10"],
        vec!["leave-group", "12"],
        vec!["leave-channel", "12"],
        vec!["search-messages", "hello", "--chat-id", "12"],
        vec!["delete-message", "42", "--for-all"],
        vec!["edit-message", "42", "Fixed"],
        vec!["forward-message", "42", "12"],
        vec!["export-file", "42", "out.txt"],
        vec!["download-file", "42", "out.txt", "--timeout", "1"],
        vec!["listen", "--auto-reply", "Hello"],
        vec!["listen", "--on-message", "./hook", "--hook-timeout", "2"],
        vec!["backup", "./backups", "--passphrase-file", "./secret"],
    ] {
        parse(&input).unwrap().validate().unwrap();
    }
    for input in [
        vec!["rename-group", "12", " "],
        vec!["search-messages", " "],
        vec!["search-messages", "x", "--chat-id", "0"],
        vec!["forward-message", "42", "bad"],
        vec!["remove-from-group", "12", "bad"],
        vec!["listen", "--on-message", "hook", "--auto-reply", "x"],
        vec!["listen", "--hook-timeout", "2"],
        vec!["listen", "--on-message", "hook", "--hook-timeout", "0"],
        vec!["status", "--for-all"],
        vec!["delete-message", "0"],
        vec!["download-file", "42", "out", "--timeout", "0"],
        vec!["restore", "/no/such/backup.tar"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err(), "{input:?}");
    }
    for input in [
        vec!["status", "--json=true"],
        vec!["listen", "--json", "--json"],
    ] {
        assert!(parse(&input).is_err());
    }
}

#[test]
fn json_encoder_preserves_unicode_and_escapes_controls() {
    use crate::output::{Value, encode, object};
    let value = object(vec![
        ("text", "\"\\\n\r\t\0🌍".into()),
        ("id", 42u32.into()),
        ("ok", true.into()),
        ("nothing", Value::Null),
        ("items", Value::Array(vec!["a".into(), "b".into()])),
    ]);
    assert_eq!(
        encode(&value),
        "{\"text\":\"\\\"\\\\\\n\\r\\t\\u0000🌍\",\"id\":42,\"ok\":true,\"nothing\":null,\"items\":[\"a\",\"b\"]}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn management_handlers_update_groups_channels_and_membership() {
    use crate::handlers;
    use deltachat::{constants::Chattype, contact::ContactId};
    let directory =
        std::env::temp_dir().join(format!("sigma-management-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let context = offline_identity(&directory, "owner").await;
    let peer = offline_identity(&directory, "peer").await;
    let person = offline_key_contact(&context, &peer).await;
    let group = chat::create_group(&context, "Team").await.unwrap();
    let group_id = group.to_u32().to_string();
    let contact_id = person.to_u32().to_string();
    handlers::add_to_group::add_member(&context, group, person)
        .await
        .unwrap();
    handlers::rename_group::handle(
        &context,
        &parse(&["rename-group", &group_id, "Renamed team"]).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(
        Chat::load_from_db(&context, group)
            .await
            .unwrap()
            .get_name(),
        "Renamed team"
    );
    handlers::group_members::handle(&context, &parse(&["group-members", &group_id]).unwrap())
        .await
        .unwrap();
    handlers::group_link::handle(&context, &parse(&["group-link", &group_id]).unwrap())
        .await
        .unwrap();
    handlers::remove_from_group::handle(
        &context,
        &parse(&["remove-from-group", &group_id, &contact_id]).unwrap(),
    )
    .await
    .unwrap();
    assert!(
        !chat::is_contact_in_chat(&context, group, person)
            .await
            .unwrap()
    );
    let channel = chat::create_broadcast(&context, "News".to_owned())
        .await
        .unwrap();
    let channel_id = channel.to_u32().to_string();
    handlers::rename_channel::handle(
        &context,
        &parse(&["rename-channel", &channel_id, "Renamed news"]).unwrap(),
    )
    .await
    .unwrap();
    handlers::set_channel_description::handle(
        &context,
        &parse(&["set-channel-description", &channel_id, "Updates"]).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(
        chat::get_chat_description(&context, channel).await.unwrap(),
        "Updates"
    );
    let photo = directory.join("photo.png");
    std::fs::write(
        &photo,
        include_bytes!(concat!(env!("OUT_DIR"), "/photo.png")),
    )
    .unwrap();
    handlers::set_channel_photo::handle(
        &context,
        &parse(&["set-channel-photo", &channel_id, photo.to_str().unwrap()]).unwrap(),
    )
    .await
    .unwrap();
    handlers::set_channel_photo::handle(
        &context,
        &parse(&["set-channel-photo", &channel_id, ""]).unwrap(),
    )
    .await
    .unwrap();
    assert!(
        crate::shared::managed_chat(&context, channel.to_u32(), Chattype::Group)
            .await
            .is_err()
    );
    handlers::leave_group::handle(&context, &parse(&["leave-group", &group_id]).unwrap())
        .await
        .unwrap();
    assert!(
        !chat::is_contact_in_chat(&context, group, ContactId::SELF)
            .await
            .unwrap()
    );
    assert!(
        handlers::rename_group::handle(
            &context,
            &parse(&["rename-group", &group_id, "No"]).unwrap()
        )
        .await
        .is_err()
    );
    drop(peer);
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn edit_forward_search_delete_and_export_are_backed_by_core() {
    use crate::handlers;
    use deltachat::message::{Message, MsgId};
    let directory =
        std::env::temp_dir().join(format!("sigma-message-tools-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let context = offline_identity(&directory, "owner").await;
    let peer = offline_identity(&directory, "peer").await;
    let contact = offline_key_contact(&context, &peer).await;
    let direct = ChatId::create_for_contact(&context, contact).await.unwrap();
    let group = chat::create_group(&context, "Forward target")
        .await
        .unwrap();
    let mut original = Message::new_text("Searchable original".to_owned());
    let id = chat::send_msg(&context, direct, &mut original)
        .await
        .unwrap();
    let id_text = id.to_u32().to_string();
    handlers::edit_message::handle(
        &context,
        &parse(&["edit-message", &id_text, "Searchable revised"]).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(
        Message::load_from_db(&context, id)
            .await
            .unwrap()
            .get_text(),
        "Searchable revised"
    );
    assert!(
        context
            .search_msgs(Some(direct), "revised")
            .await
            .unwrap()
            .contains(&id)
    );
    handlers::forward_message::handle(
        &context,
        &parse(&["forward-message", &id_text, &group.to_u32().to_string()]).unwrap(),
    )
    .await
    .unwrap();
    let items = chat::get_chat_msgs(&context, group).await.unwrap();
    assert!(
        items
            .into_iter()
            .any(|item| matches!(item, chat::ChatItem::Message { .. }))
    );
    let source = directory.join("source.txt");
    std::fs::write(&source, b"attachment bytes").unwrap();
    let mut attachment = attachment_message(&context, &source, None, Viewtype::File).unwrap();
    let attachment_id = chat::send_msg(&context, direct, &mut attachment)
        .await
        .unwrap();
    let export = directory.join("export.txt");
    let arguments = parse(&[
        "export-file",
        &attachment_id.to_u32().to_string(),
        export.to_str().unwrap(),
    ])
    .unwrap();
    handlers::export_file::handle(&context, &arguments)
        .await
        .unwrap();
    assert_eq!(std::fs::read(&export).unwrap(), b"attachment bytes");
    assert!(
        handlers::export_file::handle(&context, &arguments)
            .await
            .is_err()
    );
    assert!(
        crate::shared::download_message(&context, attachment_id, Some("1"))
            .await
            .is_ok()
    );
    handlers::delete_message::handle(&context, &parse(&["delete-message", &id_text]).unwrap())
        .await
        .unwrap();
    assert!(Message::load_from_db(&context, id).await.is_err());
    assert!(
        crate::shared::download_message(&context, MsgId::new(u32::MAX), Some("1"))
            .await
            .is_err()
    );
    drop(attachment);
    drop(original);
    drop(peer);
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn encrypted_backup_restore_round_trip_refuses_existing_account() {
    use crate::{account, handlers};
    let directory = std::env::temp_dir().join(format!("sigma-backup-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let context = offline_identity(&directory, "owner").await;
    let id = chat::create_group(&context, "Preserved group")
        .await
        .unwrap();
    let source = directory.join("attachment.txt");
    std::fs::write(&source, b"preserved backup attachment").unwrap();
    let mut attachment = attachment_message(&context, &source, None, Viewtype::File).unwrap();
    let saved = ChatId::create_for_contact(&context, deltachat::contact::ContactId::SELF)
        .await
        .unwrap();
    let attachment_id = chat::send_msg(&context, saved, &mut attachment)
        .await
        .unwrap();

    let secret = directory.join("passphrase");
    std::fs::write(&secret, "test backup passphrase\n").unwrap();
    let backup_dir = directory.join("backups");
    handlers::backup::handle(
        &context,
        &parse(&[
            "backup",
            backup_dir.to_str().unwrap(),
            "--passphrase-file",
            secret.to_str().unwrap(),
        ])
        .unwrap(),
    )
    .await
    .unwrap();
    let backup = deltachat::imex::has_backup(&context, &backup_dir)
        .await
        .unwrap();
    let destination = directory.join("restored");
    let args = parse(&[
        "restore",
        &backup,
        "--data-path",
        destination.to_str().unwrap(),
        "--passphrase-file",
        secret.to_str().unwrap(),
    ])
    .unwrap();
    let restored = account::open(&args).await.unwrap();
    handlers::restore::handle(&restored.context, &args)
        .await
        .unwrap();
    assert_eq!(
        Chat::load_from_db(&restored.context, id)
            .await
            .unwrap()
            .get_name(),
        "Preserved group"
    );
    assert_eq!(
        restored.context.get_primary_self_addr().await.unwrap(),
        "owner@example.org"
    );
    let restored_message =
        deltachat::message::Message::load_from_db(&restored.context, attachment_id)
            .await
            .unwrap();
    assert_eq!(
        std::fs::read(restored_message.get_file(&restored.context).unwrap()).unwrap(),
        b"preserved backup attachment"
    );
    drop(restored_message);
    drop(attachment);

    drop(restored);
    assert!(account::open(&args).await.is_err());
    assert!(std::path::Path::new(&backup).is_file());
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn executable_hooks_receive_json_and_have_limits() {
    use std::os::unix::fs::PermissionsExt;
    let directory = std::env::temp_dir().join(format!("sigma-hook-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("hook with spaces");
    let write = |script: &str| {
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    };
    write("#!/bin/sh\ncat\n");
    assert_eq!(
        crate::automation::run_hook(&path, "{\"text\":\"hello\"}", 2)
            .await
            .unwrap(),
        "{\"text\":\"hello\"}\n"
    );
    write("#!/bin/sh\nexit 7\n");
    assert!(crate::automation::run_hook(&path, "{}", 2).await.is_err());
    write("#!/bin/sh\nexec head -c 70000 /dev/zero\n");
    assert!(
        crate::automation::run_hook(&path, "{}", 2)
            .await
            .unwrap_err()
            .to_string()
            .contains("64 KiB")
    );
    write("#!/bin/sh\nexec sleep 5\n");
    assert!(
        crate::automation::run_hook(&path, "{}", 1)
            .await
            .unwrap_err()
            .to_string()
            .contains("timed out")
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn webxdc_update_commands_validate_cursors_and_ids() {
    for input in [
        vec![
            "send-webxdc-update",
            "26",
            r#"{"payload":{"action":"click"}}"#,
        ],
        vec!["webxdc-updates", "26", "--after", "0"],
        vec![
            "listen-webxdc-updates",
            "26",
            "--after",
            "4294967295",
            "--json",
        ],
    ] {
        parse(&input).unwrap().validate().unwrap();
    }
    for input in [
        vec!["send-webxdc-update", "0", "{}"],
        vec!["send-webxdc-update", "26"],
        vec!["webxdc-updates", "26", "--after", "-1"],
        vec!["webxdc-updates", "26", "--after", "4294967296"],
        vec!["listen-webxdc-updates", "not-an-id"],
        vec!["listen", "--after", "1"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err(), "{input:?}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn webxdc_state_updates_are_persistent_filtered_and_validated_by_core() {
    use crate::handlers::{send_webxdc, send_webxdc_update, webxdc_updates};
    use deltachat::{
        contact::ContactId,
        message::{Message, MsgId},
    };
    let directory = std::env::temp_dir().join(format!("sigma-xdc-updates-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let context = offline_identity(&directory, "app-owner").await;
    let app = directory.join("button.xdc");
    std::fs::write(
        &app,
        include_bytes!(concat!(env!("OUT_DIR"), "/button-app.xdc")),
    )
    .unwrap();
    let chat = ChatId::create_for_contact(&context, ContactId::SELF)
        .await
        .unwrap();
    let mut message = send_webxdc::prepare(&context, &app, None).await.unwrap();
    let id = chat::send_msg(&context, chat, &mut message).await.unwrap();
    let sid = id.to_u32().to_string();
    let events = context.get_event_emitter();
    let args = parse(&[
        "send-webxdc-update",
        &sid,
        r#"{"payload":{"action":"button_clicked","text":"Grüße 🌍"},"summary":"clicked"}"#,
    ])
    .unwrap();
    send_webxdc_update::handle(&context, &args).await.unwrap();
    let (first, cursor) = webxdc_updates::updates(&context, id, 0).await.unwrap();
    assert!(cursor > 0);
    assert!(first.contains("button_clicked") && first.contains("Grüße 🌍"));
    assert!(!first.contains("\"uid\""));
    assert_eq!(
        webxdc_updates::updates(&context, id, cursor).await.unwrap(),
        ("[]".into(), cursor)
    );
    let event = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            if let Some(e) = events.recv().await
                && let deltachat::EventType::WebxdcStatusUpdate { msg_id, .. } = e.typ
                && msg_id == id
            {
                break;
            }
        }
    })
    .await;
    assert!(event.is_ok(), "core must emit a status update event");
    context
        .send_webxdc_status_update(id, r#"{"payload":{"text":"From CLI"}}"#)
        .await
        .unwrap();
    let (second, later) = webxdc_updates::updates(&context, id, cursor).await.unwrap();
    assert!(later > cursor && second.contains("From CLI") && !second.contains("button_clicked"));
    assert!(
        context
            .send_webxdc_status_update(id, "invalid json")
            .await
            .is_err()
    );
    assert!(context.send_webxdc_status_update(id, "{}").await.is_err());
    assert!(webxdc_updates::instance(&context, "999999").await.is_err());
    let mut text = Message::new_text("not an app".into());
    let text_id = chat::send_msg(&context, chat, &mut text).await.unwrap();
    assert!(
        webxdc_updates::instance(&context, &text_id.to_u32().to_string())
            .await
            .is_err()
    );
    assert_eq!(
        webxdc_updates::instance(&context, &sid).await.unwrap(),
        MsgId::new(id.to_u32())
    );
    let before = webxdc_updates::updates(&context, id, 0).await.unwrap();
    drop(events);
    drop(context);
    let reopened = ContextBuilder::new(directory.join("app-owner.db"))
        .open()
        .await
        .unwrap();
    assert_eq!(
        before,
        webxdc_updates::updates(&reopened, id, 0).await.unwrap()
    );
    drop(reopened);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn realtime_command_options_and_payload_bytes() {
    use crate::realtime;
    for input in [
        vec!["send-webxdc-realtime", "83", "hello 🌍"],
        vec![
            "send-webxdc-realtime",
            "83",
            "packet.bin",
            "--file",
            "--timeout",
            "5",
        ],
        vec!["listen-webxdc-realtime", "83", "--send", "hello", "--json"],
        vec!["listen-webxdc-realtime", "83", "--send-file", "packet.bin"],
    ] {
        parse(&input).unwrap().validate().unwrap();
    }
    for input in [
        vec!["send-webxdc-realtime", "0", "hello"],
        vec!["send-webxdc-realtime", "83"],
        vec!["send-webxdc-realtime", "83", "hello", "--timeout", "0"],
        vec![
            "listen-webxdc-realtime",
            "83",
            "--send",
            "a",
            "--send-file",
            "b",
        ],
        vec!["listen-webxdc-realtime", "83", "--after", "0"],
        vec!["listen-webxdc-realtime", "83", "--file"],
        vec!["listen", "--send", "hello"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err(), "{input:?}");
    }
    assert_eq!(
        realtime::payload("Grüße 🌍", false).unwrap(),
        "Grüße 🌍".as_bytes()
    );
    assert_eq!(realtime::payload("", false).unwrap(), Vec::<u8>::new());
    assert_eq!(
        realtime::payload(&"a".repeat(realtime::MAX_BYTES), false)
            .unwrap()
            .len(),
        realtime::MAX_BYTES
    );
    assert!(realtime::payload(&"a".repeat(realtime::MAX_BYTES + 1), false).is_err());
    let file = std::env::temp_dir().join(format!("sigma-realtime-packet-{}", std::process::id()));
    std::fs::write(&file, [0u8, 255, 128, 10]).unwrap();
    let data = realtime::payload(file.to_str().unwrap(), true).unwrap();
    assert_eq!(data, [0, 255, 128, 10]);
    let fields = realtime::data_fields(deltachat::message::MsgId::new(83), &data);
    assert_eq!(
        fields.iter().find(|(key, _)| *key == "text").unwrap().1,
        crate::output::Value::Null
    );
    let encoded = crate::output::encode(&crate::output::object(fields));
    assert!(encoded.contains("\"data\":[0,255,128,10]"));
    std::fs::write(&file, vec![0; realtime::MAX_BYTES + 1]).unwrap();
    assert!(realtime::payload(file.to_str().unwrap(), true).is_err());
    std::fs::remove_file(file).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn voice_recordings_preserve_type_file_and_duration() {
    use crate::handlers::send_voice;
    use deltachat::{contact::ContactId, message::Message};
    let directory = std::env::temp_dir().join(format!("sigma-voice-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let recording = directory.join("voice.wav");
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/voice-tone.wav"));
    std::fs::write(&recording, bytes).unwrap();
    let file = recording.to_str().unwrap();
    for input in [
        vec!["send-voice", "10", file, "--duration", "1000"],
        vec!["send-voice", "12", file, "--chat", "--caption", "Voice"],
        vec![
            "send-voice",
            "https://i.delta.chat/#invite",
            file,
            "--timeout",
            "20",
        ],
    ] {
        parse(&input).unwrap().validate().unwrap();
    }
    for input in [
        vec!["send-voice", "0", file],
        vec!["send-voice", "10", file, "--duration", "-1"],
        vec!["send-voice", "10", file, "--duration", "2147483648"],
        vec!["send-voice", "10", file, "--as-text"],
        vec!["send-photo", "10", file, "--duration", "1000"],
        vec!["send-voice", "10", "missing-voice-file.ogg"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err(), "{input:?}");
    }
    let context = offline_identity(&directory, "voice-owner").await;
    let mut message =
        send_voice::prepare(&context, &recording, Some("Voice caption"), Some("1000")).unwrap();
    assert_eq!(message.get_viewtype(), Viewtype::Voice);
    assert_eq!(message.get_duration(), 1000);
    let chat = ChatId::create_for_contact(&context, ContactId::SELF)
        .await
        .unwrap();
    let id = chat::send_msg(&context, chat, &mut message).await.unwrap();
    let loaded = Message::load_from_db(&context, id).await.unwrap();
    assert_eq!(loaded.get_viewtype(), Viewtype::Voice);
    assert_eq!(loaded.get_duration(), 1000);
    assert_eq!(loaded.get_text(), "Voice caption");
    assert_eq!(
        std::fs::read(loaded.get_file(&context).unwrap()).unwrap(),
        bytes
    );
    assert_eq!(
        send_voice::prepare(&context, &recording, None, None)
            .unwrap()
            .get_duration(),
        0
    );
    assert!(send_voice::prepare(&context, &recording, None, Some("-1")).is_err());
    std::fs::write(&recording, []).unwrap();
    assert!(send_voice::prepare(&context, &recording, None, None).is_err());
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn named_app_cli_accepts_safe_names_and_event_cursors() {
    for input in [
        vec!["app-list"],
        vec!["app-bind", "demo", "95"],
        vec!["app-start", "demo"],
        vec!["app-info", "demo"],
        vec!["app-stop", "demo"],
        vec!["app-run", "demo"],
        vec!["app-send", "demo", "hello", "--file"],
        vec!["app-update", "demo", "{\"payload\":1}"],
        vec!["app-events", "demo", "--after", "4294967296", "--json"],
    ] {
        parse(&input).unwrap().validate().unwrap();
    }
    for input in [
        vec!["app-bind", "demo", "0"],
        vec!["app-start", "../escape"],
        vec!["app-events", "demo", "--after", "-1"],
        vec!["app-events", "demo", "--after", "9223372036854775808"],
        vec!["app-info", "demo", "--file"],
        vec!["app-update", "demo"],
        vec!["app-send", "demo", "hi", "--timeout", "5"],
        vec!["send-webxdc", "10", "missing.xdc", "--app", "../escape"],
    ] {
        assert!(parse(&input).unwrap().validate().is_err(), "{input:?}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn named_apps_persist_identity_and_recover_committed_events_and_update_cursors() {
    use crate::apps::{self, App, Journal};
    use deltachat::{contact::ContactId, message::Message};
    let directory = std::env::temp_dir().join(format!("sigma-named-apps-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let context = offline_identity(&directory, "owner").await;
    let file = directory.join("app.xdc");
    std::fs::write(
        &file,
        include_bytes!(concat!(env!("OUT_DIR"), "/button-app.xdc")),
    )
    .unwrap();
    let chat = ChatId::create_for_contact(&context, ContactId::SELF)
        .await
        .unwrap();
    let mut message = crate::handlers::send_webxdc::prepare(&context, &file, None)
        .await
        .unwrap();
    let id = chat::send_msg(&context, chat, &mut message).await.unwrap();
    let app = apps::bind(&context, &directory, "demo", id).await.unwrap();
    assert_eq!(
        App::load(&directory, "demo").unwrap().message_id,
        id.to_u32()
    );
    assert!(apps::bind(&context, &directory, "demo", id).await.is_err());
    assert!(App::load(&directory, "../outside").is_err());
    let mut text = Message::new_text("not an app".into());
    let text_id = chat::send_msg(&context, chat, &mut text).await.unwrap();
    assert!(
        apps::bind(&context, &directory, "text", text_id)
            .await
            .is_err()
    );
    let mut journal = Journal::open(&directory, &app).unwrap();
    assert_eq!(
        journal
            .append(
                "realtime_received",
                vec![("text", "hello 🌍\n".into())],
                None
            )
            .unwrap(),
        1
    );
    assert_eq!(
        journal
            .append(
                "webxdc_updates",
                vec![("last_serial", 7u32.into())],
                Some("[{\"serial\":7,\"payload\":1}]")
            )
            .unwrap(),
        2
    );
    drop(journal);
    use std::io::Write;
    let path = directory.join("apps/demo/events.jsonl");
    let committed = std::fs::read(&path).unwrap();
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    file.write_all(b"{\"seq\":3,\"partial\":").unwrap();
    drop(file);
    let mut recovered = Journal::open(&directory, &App::load(&directory, "demo").unwrap()).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), committed);
    assert_eq!(recovered.last_serial, 7);
    assert_eq!(recovered.append("worker_started", vec![], None).unwrap(), 3);
    drop(recovered);
    for line in std::fs::read_to_string(&path).unwrap().lines() {
        let value = deltachat::webxdc::StatusUpdateItem {
            payload: line.parse().unwrap(),
            ..Default::default()
        };
        assert_eq!(value.payload["app"].as_str(), Some("demo"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(directory.join("apps/demo"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            std::fs::metadata(directory.join("apps/demo/app.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn bot_and_group_photos_use_core_and_reject_other_chat_types() {
    use crate::handlers::{set_group_photo, set_profile_photo};
    use deltachat::{config::Config, contact::ContactId};
    let directory = std::env::temp_dir().join(format!("sigma-avatar-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let photo = directory.join("photo.png");
    std::fs::write(
        &photo,
        include_bytes!(concat!(env!("OUT_DIR"), "/photo.png")),
    )
    .unwrap();
    let path = photo.to_str().unwrap();
    let context = offline_identity(&directory, "owner").await;
    let group = chat::create_group(&context, "Photo group").await.unwrap();
    let group_id = group.to_u32().to_string();
    let args = parse(&["set-group-photo", &group_id, path]).unwrap();
    args.validate().unwrap();
    set_group_photo::handle(&context, &args).await.unwrap();
    let saved = Chat::load_from_db(&context, group)
        .await
        .unwrap()
        .get_profile_image(&context)
        .await
        .unwrap()
        .unwrap();
    assert!(saved.is_file());
    set_group_photo::handle(
        &context,
        &parse(&["set-group-photo", &group_id, ""]).unwrap(),
    )
    .await
    .unwrap();
    assert!(
        Chat::load_from_db(&context, group)
            .await
            .unwrap()
            .get_profile_image(&context)
            .await
            .unwrap()
            .is_none()
    );
    let direct = ChatId::create_for_contact(&context, ContactId::SELF)
        .await
        .unwrap();
    assert!(
        set_group_photo::handle(
            &context,
            &parse(&["set-group-photo", &direct.to_u32().to_string(), path]).unwrap()
        )
        .await
        .is_err()
    );
    let channel = chat::create_broadcast(&context, "Photo channel".into())
        .await
        .unwrap();
    assert!(
        set_group_photo::handle(
            &context,
            &parse(&["set-group-photo", &channel.to_u32().to_string(), path]).unwrap()
        )
        .await
        .is_err()
    );
    set_profile_photo::handle(&context, &parse(&["set-profile-photo", path]).unwrap())
        .await
        .unwrap();
    let saved = context
        .get_config(Config::Selfavatar)
        .await
        .unwrap()
        .unwrap();
    assert!(std::path::Path::new(&saved).is_file());
    set_profile_photo::handle(&context, &parse(&["set-profile-photo", ""]).unwrap())
        .await
        .unwrap();
    assert!(
        context
            .get_config(Config::Selfavatar)
            .await
            .unwrap()
            .is_none()
    );
    assert!(crate::shared::avatar_path(directory.to_str().unwrap()).is_err());
    assert!(
        parse(&["set-group-photo", "0", path])
            .unwrap()
            .validate()
            .is_err()
    );
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn app_hooks_save_config_recover_pending_and_validate_actions() {
    use crate::{
        app_hooks,
        apps::{App, Journal},
    };
    use std::os::unix::fs::PermissionsExt;
    let directory = std::env::temp_dir().join(format!("sigma-app-hooks-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let context = offline_identity(&directory, "hook-owner").await;
    let app_file = directory.join("app.xdc");
    std::fs::write(
        &app_file,
        include_bytes!(concat!(env!("OUT_DIR"), "/button-app.xdc")),
    )
    .unwrap();
    let chat_id = ChatId::create_for_contact(&context, deltachat::contact::ContactId::SELF)
        .await
        .unwrap();
    let mut message = crate::handlers::send_webxdc::prepare(&context, &app_file, None)
        .await
        .unwrap();
    let id = chat::send_msg(&context, chat_id, &mut message)
        .await
        .unwrap();
    let mut app = crate::apps::bind(&context, &directory, "hooked", id)
        .await
        .unwrap();
    let hook = directory.join("hook.py");
    std::fs::write(&hook, "#!/usr/bin/env python3\nimport sys,json\ne=json.load(sys.stdin)\nprint(json.dumps({'update': {'payload': {'hook_seq':e['seq']}}}))\n").unwrap();
    let args = parse(&[
        "app-start",
        "hooked",
        "--on-event",
        hook.to_str().unwrap(),
        "--hook-timeout",
        "2",
    ])
    .unwrap();
    args.validate().unwrap();
    assert!(app_hooks::configure(&mut app, &directory, &args).is_err());
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o700)).unwrap();
    app_hooks::configure(&mut app, &directory, &args).unwrap();
    let saved = App::load(&directory, "hooked").unwrap();
    assert_eq!(saved.hook_timeout, 2);
    assert!(saved.hook_path.unwrap().is_absolute());
    let mut journal = Journal::open(&directory, &app).unwrap();
    let first = journal
        .append("realtime_received", vec![("text", "button".into())], None)
        .unwrap();
    let input = journal.last_line.clone();
    let second = journal
        .append("realtime_received", vec![("text", "second".into())], None)
        .unwrap();
    journal
        .append(
            "hook_finished",
            vec![("event_seq", second.into()), ("success", false.into())],
            None,
        )
        .unwrap();
    let events_file = directory.join("apps/hooked/events.jsonl");
    let pending = app_hooks::recover(&events_file, &app.hooks).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].seq, first);
    let output = app_hooks::spawn(hook, input, directory.join("apps/hooked"), 2)
        .await
        .unwrap()
        .unwrap();
    app_hooks::actions(&context, id, &output, false)
        .await
        .unwrap();
    assert!(
        crate::handlers::webxdc_updates::updates(&context, id, 0)
            .await
            .unwrap()
            .0
            .contains("hook_seq")
    );
    for invalid in [
        "not json",
        "[]",
        "{\"unknown\":1}",
        "{\"send\":1}",
        "{\"update\":{}}",
        "{\"send\":\"hello\"}",
    ] {
        assert!(
            app_hooks::actions(&context, id, invalid, false)
                .await
                .is_err(),
            "{invalid}"
        );
    }
    app_hooks::actions(&context, id, "", false).await.unwrap();
    app_hooks::configure(
        &mut app,
        &directory,
        &parse(&["app-start", "hooked", "--no-hook"]).unwrap(),
    )
    .unwrap();
    assert!(App::load(&directory, "hooked").unwrap().hook_path.is_none());
    for args in [
        vec!["app-start", "hooked", "--on-event", "/tmp/a", "--no-hook"],
        vec!["app-run", "hooked", "--hook-timeout", "2"],
        vec![
            "app-start",
            "hooked",
            "--on-event",
            "/tmp/a",
            "--hook-timeout",
            "0",
        ],
    ] {
        assert!(parse(&args).unwrap().validate().is_err());
    }
    drop(journal);
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn multiple_hooks_select_types_and_payloads_and_recover_independently() {
    use crate::app_hooks::{self, Hook};
    let hook = |id: &str, event: &str, filter: Option<&str>| Hook {
        id: id.into(),
        path: "/tmp/hook".into(),
        event: event.into(),
        match_json: filter.map(str::to_owned),
        timeout: 5,
        after_seq: 0,
    };
    let rt = hook(
        "live",
        "realtime_received",
        Some(r#"{"type":"button_clicked","details":{"button":"ok"}}"#),
    );
    let updates = hook("saved", "webxdc_updates", Some(r#"{"action":"submit"}"#));
    let packet = r#"{"seq":1,"type":"realtime_received","text":"{\"type\":\"button_clicked\",\"details\":{\"button\":\"ok\",\"extra\":1}}"}"#;
    let selected = app_hooks::select(&rt, 1, packet).unwrap().unwrap();
    assert_eq!(selected.hook.id, "live");
    assert!(selected.input.contains("hook_id"));
    assert!(app_hooks::select(&updates, 1, packet).unwrap().is_none());
    assert!(
        app_hooks::select(
            &rt,
            2,
            r#"{"seq":2,"type":"realtime_received","text":"not JSON"}"#
        )
        .unwrap()
        .is_none()
    );
    let batch = r#"{"seq":3,"type":"webxdc_updates","updates":[{"serial":1,"payload":{"action":"ignore"}},{"serial":2,"payload":{"action":"submit","value":7}}]}"#;
    let selected = app_hooks::select(&updates, 3, batch).unwrap().unwrap();
    assert!(!selected.input.contains("ignore"));
    assert!(selected.input.contains("submit"));
    let second = hook("second", "realtime_received", None);
    let path = std::env::temp_dir().join(format!("sigma-multi-hook-{}.jsonl", std::process::id()));
    std::fs::write(&path,format!("{packet}\n{{\"seq\":2,\"type\":\"hook_finished\",\"hook_id\":\"live\",\"event_seq\":1}}\n")).unwrap();
    let pending = app_hooks::recover(&path, &[rt.clone(), second]).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].hook.id, "second");
    let mut later = rt;
    later.after_seq = 1;
    assert!(app_hooks::select(&later, 1, packet).unwrap().is_none());
    std::fs::remove_file(path).unwrap();
    let old_root =
        std::env::temp_dir().join(format!("sigma-hook-migration-{}", std::process::id()));
    std::fs::create_dir_all(old_root.join("apps/old")).unwrap();
    std::fs::write(old_root.join("apps/old/app.json"), r#"{"name":"old","message_id":10,"chat_id":11,"last_serial":0,"hook_path":"/tmp/legacy-hook","hook_timeout":9,"hook_after_seq":5}"#).unwrap();
    let migrated = crate::apps::App::load(&old_root, "old").unwrap();
    assert_eq!(migrated.hooks.len(), 1);
    assert_eq!(migrated.hooks[0].id, "default");
    assert_eq!(migrated.hooks[0].after_seq, 5);
    migrated.save(&old_root).unwrap();
    assert_eq!(
        crate::apps::App::load(&old_root, "old")
            .unwrap()
            .hooks
            .len(),
        1
    );
    std::fs::remove_dir_all(old_root).unwrap();
    assert!(app_hooks::validate_selector("wrong", None).is_err());
    assert!(app_hooks::validate_selector("all", Some("[]")).is_err());
    for args in [
        vec![
            "app-hook-add",
            "demo",
            "live",
            "/tmp/hook",
            "--event",
            "realtime_received",
            "--match",
            "{\"type\":\"button_clicked\"}",
            "--hook-timeout",
            "2",
        ],
        vec!["app-hook-list", "demo"],
        vec!["app-hook-remove", "demo", "live"],
    ] {
        parse(&args).unwrap().validate().unwrap();
    }
}

#[test]
fn typed_hooks_classify_messages_commands_and_realtime_requirement() {
    use crate::app_hooks::{self, Hook};
    let mk = |event: &str| Hook {
        id: event.into(),
        path: "/tmp/hook".into(),
        event: app_hooks::normalize(event).unwrap().into(),
        match_json: None,
        timeout: 3,
        after_seq: 0,
    };
    assert!(!app_hooks::needs_realtime(&[]));
    assert!(!app_hooks::needs_realtime(&[
        mk("update"),
        mk("message"),
        mk("command")
    ]));
    assert!(app_hooks::needs_realtime(&[mk("realtime")]));
    assert!(app_hooks::needs_realtime(&[mk("all")]));
    assert_eq!(
        app_hooks::command("/ping hello world"),
        Some(("/ping".into(), "hello world".into()))
    );
    assert!(app_hooks::command("ordinary message").is_none());
    assert!(app_hooks::command("/").is_none());
    let args = parse(&[
        "app-hook-add",
        "demo",
        "ping",
        "/tmp/hook",
        "--type",
        "command",
        "--command",
        "/ping",
    ])
    .unwrap();
    args.validate().unwrap();
    let (event, filter) = app_hooks::selectors(&args).unwrap();
    assert_eq!(event, "command_received");
    let mut command = mk("command");
    command.match_json = filter;
    let input = r#"{"seq":1,"type":"command_received","command":"/ping","args":"hello","text":"/ping hello"}"#;
    assert!(app_hooks::select(&command, 1, input).unwrap().is_some());
    assert!(
        app_hooks::select(&mk("message"), 1, input)
            .unwrap()
            .is_none()
    );
    assert!(app_hooks::select(&mk("all"), 1, input).unwrap().is_none());
    for input in [
        vec!["app-hook-add", "demo", "h", "/tmp/hook"],
        vec![
            "app-hook-add",
            "demo",
            "h",
            "/tmp/hook",
            "--type",
            "update",
            "--command",
            "/ping",
        ],
        vec![
            "app-hook-add",
            "demo",
            "h",
            "/tmp/hook",
            "--type",
            "command",
            "--command",
            "ping",
        ],
        vec![
            "app-hook-add",
            "demo",
            "h",
            "/tmp/hook",
            "--type",
            "message",
            "--event",
            "message_received",
        ],
    ] {
        assert!(parse(&input).unwrap().validate().is_err(), "{input:?}");
    }
}

#[test]
fn chat_command_hooks_require_chat_and_exact_slash_command() {
    for a in [
        vec![
            "chat-hook-add",
            "13",
            "ping",
            "./hook",
            "--command",
            "/ping",
        ],
        vec!["chat-hook-list", "13"],
        vec!["chat-hook-remove", "13", "ping"],
    ] {
        parse(&a).unwrap().validate().unwrap();
    }
    for a in [
        vec!["chat-hook-add", "13", "ping", "./hook"],
        vec!["chat-hook-add", "13", "ping", "./hook", "--command", "ping"],
        vec![
            "chat-hook-add",
            "13",
            "ping",
            "./hook",
            "--command",
            "/ping extra",
        ],
        vec!["chat-hook-list", "app-name"],
        vec!["chat-hook-list", "1"],
    ] {
        assert!(parse(&a).and_then(|a| a.validate()).is_err());
    }
}

#[test]
fn contact_alias_arguments() {
    let invite = "https://i.delta.chat/#fingerprint&i=token&s=auth";
    for args in [
        vec!["create-contact", "me", invite],
        vec!["send-message", "me", "Hello"],
        vec!["create-contact", invite],
    ] {
        parse(&args).unwrap().validate().unwrap();
    }
    for alias in [
        "",
        "123",
        "../me",
        "my name",
        "me\nother",
        "99999999999999999999",
    ] {
        assert!(
            parse(&["create-contact", alias, invite])
                .unwrap()
                .validate()
                .is_err()
        );
    }
    assert!(
        parse(&["send-message", "me", "Hello", "--chat"])
            .unwrap()
            .validate()
            .is_err()
    );
    assert!(
        parse(&["send-message", "0", "Hello"])
            .unwrap()
            .validate()
            .is_err()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contact_aliases_persist_and_do_not_cross_accounts() {
    let directory = std::env::temp_dir().join(format!("sigma-alias-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let database = directory.join("account.db");
    let context = ContextBuilder::new(database.clone()).open().await.unwrap();
    let id = Contact::create(&context, "A display name", "alias-test@example.org")
        .await
        .unwrap();
    crate::aliases::save(&context, "me", id).await.unwrap();
    assert!(crate::aliases::save(&context, "me", id).await.is_err());
    assert_eq!(crate::aliases::resolve(&context, "me").await.unwrap(), id);
    assert_eq!(
        crate::aliases::resolve(&context, &id.to_u32().to_string())
            .await
            .unwrap(),
        id
    );
    drop(context);
    let context = ContextBuilder::new(database).open().await.unwrap();
    assert_eq!(crate::aliases::resolve(&context, "me").await.unwrap(), id);
    assert!(crate::aliases::resolve(&context, "ME").await.is_err());
    let other = directory.join("other");
    std::fs::create_dir_all(&other).unwrap();
    let second = ContextBuilder::new(other.join("account.db"))
        .open()
        .await
        .unwrap();
    assert!(crate::aliases::resolve(&second, "me").await.is_err());
    std::fs::write(
        directory.join("contact-aliases.tsv"),
        format!("me\t{}\twrong@example.org\n", id.to_u32()),
    )
    .unwrap();
    assert!(crate::aliases::resolve(&context, "me").await.is_err());
    drop(second);
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn init_profile_defaults_and_overrides() {
    use deltachat::config::Config;
    let directory =
        std::env::temp_dir().join(format!("sigma-default-profile-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let context = ContextBuilder::new(directory.join("account.db"))
        .open()
        .await
        .unwrap();
    let args = parse(&["init"]).unwrap();
    crate::handlers::init::configure_profile(&context, &args, &directory)
        .await
        .unwrap();
    assert_eq!(
        context
            .get_config(Config::Displayname)
            .await
            .unwrap()
            .as_deref(),
        Some("Sigma")
    );
    let avatar = context
        .get_config(Config::Selfavatar)
        .await
        .unwrap()
        .unwrap();
    assert!(std::path::Path::new(&avatar).is_file());
    context
        .set_config(Config::Displayname, Some("Custom bot"))
        .await
        .unwrap();
    crate::handlers::init::configure_profile(&context, &args, &directory)
        .await
        .unwrap();
    assert_eq!(
        context
            .get_config(Config::Displayname)
            .await
            .unwrap()
            .as_deref(),
        Some("Custom bot")
    );
    assert_eq!(
        context
            .get_config(Config::Selfavatar)
            .await
            .unwrap()
            .unwrap(),
        avatar
    );
    let photo = directory.join("custom.png");
    std::fs::write(
        &photo,
        include_bytes!(concat!(env!("OUT_DIR"), "/photo.png")),
    )
    .unwrap();
    let args = parse(&[
        "init",
        "--name",
        "Override",
        "--profile-photo-address",
        photo.to_str().unwrap(),
    ])
    .unwrap();
    crate::handlers::init::configure_profile(&context, &args, &directory)
        .await
        .unwrap();
    assert_eq!(
        context
            .get_config(Config::Displayname)
            .await
            .unwrap()
            .as_deref(),
        Some("Override")
    );
    assert_ne!(
        context
            .get_config(Config::Selfavatar)
            .await
            .unwrap()
            .unwrap(),
        avatar
    );
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn agent_connect_arguments_require_an_explicit_contact() {
    for input in [
        vec!["agent-connect", "grok", "--to", "me"],
        vec![
            "agent-connect",
            "grok",
            "--to",
            "me",
            "--model",
            "test-model",
        ],
        vec![
            "agent-connect",
            "grok",
            "--to",
            "42",
            "--exec",
            "./adapter",
            "--hook-timeout",
            "60",
        ],
    ] {
        parse(&input).unwrap().validate().unwrap();
    }
    for input in [
        vec!["agent-connect", "grok", "--to", "me", "--model", " "],
        vec![
            "agent-connect",
            "other",
            "--to",
            "me",
            "--model",
            "test-model",
        ],
        vec![
            "agent-connect",
            "grok",
            "--to",
            "me",
            "--exec",
            "./adapter",
            "--model",
            "test-model",
        ],
        vec!["listen", "--model", "test-model"],
        vec!["agent-connect", "grok", "--to", "0"],
        vec!["agent-connect", "../grok", "--to", "me"],
        vec!["agent-connect", "grok", "--to", "a b"],
        vec!["agent-connect", "grok", "--to", "me", "--hook-timeout", "0"],
        vec!["agent-connect", "grok", "--to", "me", "--auto-reply", "hi"],
        vec!["listen", "--to", "me"],
        vec!["listen", "--exec", "./adapter"],
    ] {
        assert!(
            parse(&input).and_then(|a| a.validate()).is_err(),
            "{input:?}"
        );
    }
    assert!(
        crate::handlers::agent_connect::executable("sigma-no-such-agent-executable", None).is_err()
    );
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn agent_adapter_receives_plain_text_and_queues_a_quoted_reply() {
    use crate::handlers::agent_connect;
    use deltachat::{contact::ContactId, message::Message};
    use std::os::unix::fs::PermissionsExt;
    let directory = std::env::temp_dir().join(format!("sigma-a2a-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let context = offline_identity(&directory, "a2a-owner").await;
    let chat = ChatId::create_for_contact(&context, ContactId::SELF)
        .await
        .unwrap();
    let mut original = Message::new_text("Grüße 🌍\n$(touch should-not-exist)".into());
    let original_id = chat::send_msg(&context, chat, &mut original).await.unwrap();
    let original = Message::load_from_db(&context, original_id).await.unwrap();
    assert!(
        !crate::agent_guard::allowed(&context, &directory, ContactId::new(42), chat, &original)
            .await
            .unwrap()
    );
    let path = directory.join("agent with spaces");
    std::fs::write(&path, "#!/bin/sh\ncat\n").unwrap();
    assert!(agent_connect::executable("grok", path.to_str()).is_err());
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    let executable = agent_connect::executable("grok", path.to_str()).unwrap();
    let id = agent_connect::reply(
        &context,
        &executable,
        2,
        &agent_connect::ReplySettings {
            native: None,
            model: None,
            system_prompt: None,
            session_dir: None,
            actions: None,
            import_history: true,
        },
        &original,
    )
    .await
    .unwrap()
    .unwrap();
    let reply = Message::load_from_db(&context, id).await.unwrap();
    assert_eq!(reply.get_text(), format!("{}\n", original.get_text()));
    assert_eq!(reply.get_chat_id(), chat);
    assert_eq!(
        reply.parent(&context).await.unwrap().unwrap().get_id(),
        original_id
    );
    std::fs::write(
        &path,
        "#!/bin/sh\ncat >/dev/null\nprintf '%s' 'internal commentary [SIGMA_NO_REPLY]'\n",
    )
    .unwrap();
    let suppressed = agent_connect::reply(
        &context,
        &executable,
        2,
        &agent_connect::ReplySettings {
            native: None,
            model: None,
            system_prompt: None,
            session_dir: None,
            actions: None,
            import_history: true,
        },
        &original,
    )
    .await
    .unwrap();
    assert!(suppressed.is_none());
    drop(context);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn agent_management_and_system_prompt_arguments() {
    for input in [
        vec!["agent-list"],
        vec!["agent-stop", "grok"],
        vec![
            "agent-connect",
            "grok",
            "--to",
            "me",
            "--system-prompt",
            "Be precise.\nGrüße 🌍",
        ],
        vec!["agent-connect", "grok", "--to", "me", "--foreground"],
    ] {
        parse(&input).unwrap().validate().unwrap();
    }
    for input in [
        vec!["a2a-connect", "grok", "--to", "me"],
        vec!["agent-stop"],
        vec!["agent-stop", "../grok"],
        vec!["agent-list", "--foreground"],
        vec![
            "agent-connect",
            "grok",
            "--to",
            "me",
            "--system-prompt",
            " ",
        ],
        vec![
            "agent-connect",
            "other",
            "--to",
            "me",
            "--system-prompt",
            "hello",
        ],
        vec![
            "agent-connect",
            "grok",
            "--to",
            "me",
            "--exec",
            "./adapter",
            "--system-prompt",
            "hello",
        ],
        vec![
            "agent-connect",
            "grok",
            "--to",
            "me",
            "--system-prompt",
            "hello",
            "--system-prompt-file",
            "./prompt",
        ],
        vec![
            "agent-connect",
            "grok",
            "--to",
            "me",
            "--system-prompt-file",
            "/no/such/prompt-file",
        ],
    ] {
        assert!(
            parse(&input).and_then(|a| a.validate()).is_err(),
            "{input:?}"
        );
    }
    let path = std::env::temp_dir().join(format!("sigma-agent-prompt-{}", std::process::id()));
    let args = parse(&[
        "agent-connect",
        "grok",
        "--to",
        "me",
        "--system-prompt-file",
        path.to_str().unwrap(),
    ])
    .unwrap();
    std::fs::write(&path, "Be precise.\nGrüße 🌍").unwrap();
    args.validate().unwrap();
    std::fs::write(&path, " \n").unwrap();
    assert!(args.validate().is_err());
    std::fs::remove_file(path).unwrap();
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn agent_control_lists_under_account_lock_and_stops_without_saved_pids() {
    let directory =
        std::env::temp_dir().join(format!("sigma-agent-control-{}", std::process::id()));
    std::fs::create_dir_all(directory.join("agents/grok")).unwrap();
    let socket = directory.join("agents/grok/control.sock");
    let stale = tokio::net::UnixListener::bind(&socket).unwrap();
    drop(stale);
    let args = parse(&[
        "agent-connect",
        "grok",
        "--to",
        "me",
        "--model",
        "test-model",
        "--system-prompt",
        "private instructions",
    ])
    .unwrap();
    let lock = AccountLock::acquire(directory.join("sigma.lock")).unwrap();
    let mut runtime = crate::agents::register(&directory, &args, 42)
        .await
        .unwrap();
    let raw = crate::agents::request(&directory, "grok", "status")
        .await
        .unwrap();
    assert!(raw.contains("test-model"));
    assert!(raw.contains("\"system_prompt\":true"));
    assert!(!raw.contains("private instructions"));
    let list = parse(&["agent-list", "--data-path", directory.to_str().unwrap()]).unwrap();
    crate::agents::list(&list).await.unwrap();
    let stop = parse(&[
        "agent-stop",
        "grok",
        "--data-path",
        directory.to_str().unwrap(),
    ])
    .unwrap();
    let task = tokio::spawn(async move { crate::agents::stop(&stop).await });
    tokio::time::timeout(std::time::Duration::from_secs(3), runtime.stop.changed())
        .await
        .unwrap()
        .unwrap();
    assert!(*runtime.stop.borrow());
    drop(runtime);
    drop(lock);
    task.await.unwrap().unwrap();
    assert!(!socket.exists());
    assert!(
        crate::agents::request(&directory, "grok", "status")
            .await
            .is_err()
    );
    crate::agents::list(&list).await.unwrap();
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn named_account_cli_and_dedicated_agents_validate_before_provisioning() {
    let invite = "https://i.delta.chat/#fingerprint&i=token&s=auth";
    for input in [
        vec!["account-list"],
        vec!["account-create", "my-bot", "--name", "My Bot"],
        vec!["account-default", "my-bot"],
        vec!["account-delete", "my-bot"],
        vec!["status", "--account", "my-bot"],
        vec!["agent-list", "--account", "my-bot"],
        vec![
            "agent-connect",
            "codex",
            "--account",
            "my-bot",
            "--to",
            "me",
            "--model",
            "test-model",
            "--system-prompt",
            "Be concise.",
        ],
        vec![
            "agent-connect",
            "codex",
            "--new-account",
            "codex-bot",
            "--name",
            "Codex",
            "--to",
            "me",
            "--invite",
            invite,
        ],
        vec![
            "agent-connect",
            "grok",
            "--new-account",
            "grok-bot",
            "--to",
            invite,
        ],
    ] {
        parse(&input).unwrap().validate().unwrap();
    }
    for input in [
        vec!["status", "--account", "../oops"],
        vec!["status", "--account", "bot", "--data-path", "./other"],
        vec!["account-create", "../bot"],
        vec![
            "agent-connect",
            "codex",
            "--to",
            "me",
            "--account-qr",
            "DCACCOUNT:test",
        ],
        vec!["account-list", "--account", "bot"],
        vec!["account-delete", "bot", "--data-path", "./other"],
        vec![
            "agent-connect",
            "codex",
            "--new-account",
            "legacy",
            "--to",
            invite,
        ],
        vec![
            "agent-connect",
            "codex",
            "--new-account",
            "bot",
            "--account",
            "other",
            "--to",
            invite,
        ],
        vec![
            "agent-connect",
            "codex",
            "--new-account",
            "bot",
            "--to",
            "me",
        ],
        vec!["agent-connect", "codex", "--to", "42", "--invite", invite],
        vec!["account-create", "bot", "--profile-photo", "/not/a/photo"],
        vec!["agent-connect", "codex", "--to", "me", "--name", " "],
    ] {
        assert!(
            parse(&input).and_then(|a| a.validate()).is_err(),
            "{input:?}"
        );
    }
}

#[test]
fn named_accounts_preserve_legacy_and_protect_running_and_default_accounts() {
    use crate::accounts;
    let directory =
        std::env::temp_dir().join(format!("sigma-accounts-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let home = Some(directory.as_os_str());
    let legacy = accounts::path(home, "legacy").unwrap();
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(legacy.join("account.db"), "legacy identity").unwrap();
    assert_eq!(
        accounts::resolve(&parse(&["status"]).unwrap(), home).unwrap(),
        legacy
    );
    let bot = accounts::reserve(home, "bot").unwrap();
    assert!(accounts::reserve(home, "bot").is_err());
    assert!(accounts::reserve(home, "../bot").is_err());
    assert!(accounts::reserve(home, "legacy").is_err());
    assert!(accounts::set_default(home, "bot").is_err());
    std::fs::write(bot.join("account.db"), "bot identity").unwrap();
    accounts::set_default(home, "bot").unwrap();
    assert_eq!(
        accounts::resolve(&parse(&["status"]).unwrap(), home).unwrap(),
        bot
    );
    assert_eq!(
        accounts::resolve(
            &parse(&["agent-list", "--account", "legacy"]).unwrap(),
            home
        )
        .unwrap(),
        legacy
    );
    assert_eq!(
        accounts::resolve(
            &parse(&["status", "--data-path", "./explicit"]).unwrap(),
            None
        )
        .unwrap(),
        std::path::PathBuf::from("./explicit")
    );
    assert!(accounts::delete(home, "bot").is_err());
    let lock = AccountLock::acquire(legacy.join("sigma.lock")).unwrap();
    assert!(accounts::delete(home, "legacy").is_err());
    drop(lock);
    // Managed accounts are siblings of legacy, so deleting legacy cannot erase them.
    accounts::delete(home, "legacy").unwrap();
    assert!(!legacy.exists());
    assert_eq!(
        std::fs::read_to_string(bot.join("account.db")).unwrap(),
        "bot identity"
    );
    assert!(accounts::delete(home, "../bot").is_err());
    std::fs::write(
        accounts::registry(home).unwrap().join("default-account"),
        "../invalid",
    )
    .unwrap();
    assert!(accounts::resolve(&parse(&["status"]).unwrap(), home).is_err());
    assert_eq!(
        accounts::resolve(&parse(&["status", "--account", "bot"]).unwrap(), home).unwrap(),
        bot
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        std::fs::write(
            accounts::registry(home).unwrap().join("default-account"),
            "bot",
        )
        .unwrap();
        symlink(&bot, accounts::path(home, "linked").unwrap()).unwrap();
        assert!(accounts::delete(home, "linked").is_err());
        assert!(bot.join("account.db").exists());
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn agent_alias_and_id_arguments() {
    for input in [
        vec![
            "agent-connect",
            "grok",
            "--alias",
            "researcher",
            "--to",
            "me",
        ],
        vec!["agent-connect", "grok", "--alias", "researcher"],
        vec!["agent-connect", "grok", "--agent-id", "1"],
        vec!["agent-stop", "1"],
        vec!["agent-stop", "researcher"],
    ] {
        parse(&input).unwrap().validate().unwrap();
    }
    for input in [
        vec!["agent-connect", "grok", "--alias", "../wrong", "--to", "me"],
        vec!["agent-connect", "grok", "--alias", "123", "--to", "me"],
        vec!["agent-connect", "grok", "--agent-id", "0"],
        vec!["agent-connect", "grok", "--agent-id", "x"],
        vec![
            "agent-connect",
            "grok",
            "--agent-id",
            "1",
            "--alias",
            "other",
        ],
        vec![
            "agent-connect",
            "grok",
            "--agent-id",
            "1",
            "--new-account",
            "new",
            "--to",
            "me",
        ],
        vec!["agent-stop", "0"],
        vec!["agent-list", "--alias", "researcher"],
    ] {
        assert!(
            parse(&input).and_then(|a| a.validate()).is_err(),
            "{input:?}"
        );
    }
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn agent_ids_and_aliases_persist_and_first_recipient_cannot_change() {
    use crate::agent_registry::{self, Agent};
    let root = socket_test_directory();
    std::fs::create_dir_all(&root).unwrap();
    let context = offline_identity(&root, "agent-owner").await;
    let person = Contact::create(&context, "First person", "first@example.org")
        .await
        .unwrap();
    let other = Contact::create(&context, "Other person", "other@example.org")
        .await
        .unwrap();
    crate::aliases::save(&context, "me", person).await.unwrap();
    crate::aliases::save(&context, "owner", person)
        .await
        .unwrap();
    let chat = ChatId::create_for_contact(&context, person).await.unwrap();
    let args = parse(&[
        "agent-connect",
        "grok",
        "--alias",
        "researcher",
        "--to",
        "me",
    ])
    .unwrap();
    let original = Agent::create(&root, &args, &context, person, chat)
        .await
        .unwrap();
    assert_eq!(original.id, 1);
    assert_eq!(original.alias, "researcher");
    assert!(
        Agent::create(&root, &args, &context, person, chat)
            .await
            .is_err()
    );
    let saved = agent_registry::load(&root, "researcher").unwrap().unwrap();
    assert_eq!(saved, original);
    let resumed = parse(&["agent-connect", "grok", "--agent-id", "1"]).unwrap();
    assert_eq!(
        agent_registry::selected(&root, &resumed).unwrap(),
        Some(original.clone())
    );
    assert_eq!(saved.recipient(&context, &resumed).await.unwrap(), person);
    let equivalent_alias = parse(&[
        "agent-connect",
        "grok",
        "--alias",
        "researcher",
        "--to",
        "owner",
    ])
    .unwrap();
    assert_eq!(
        saved.recipient(&context, &equivalent_alias).await.unwrap(),
        person
    );
    let retarget = parse(&[
        "agent-connect",
        "grok",
        "--alias",
        "researcher",
        "--to",
        &other.to_u32().to_string(),
    ])
    .unwrap();
    assert!(saved.recipient(&context, &retarget).await.is_err());
    let changed_invite = parse(&[
        "agent-connect",
        "grok",
        "--alias",
        "researcher",
        "--to",
        "https://i.delta.chat/#new&i=token&s=auth",
    ])
    .unwrap();
    assert!(saved.recipient(&context, &changed_invite).await.is_err());
    let provider_change = parse(&["agent-connect", "codex", "--agent-id", "1"]).unwrap();
    assert!(agent_registry::selected(&root, &provider_change).is_err());
    // Reassigning the contact alias cannot reassign the agent's bound person.
    std::fs::write(
        root.join("contact-aliases.tsv"),
        format!("me\t{}\tother@example.org\n", other.to_u32()),
    )
    .unwrap();
    assert!(saved.recipient(&context, &args).await.is_err());
    assert_eq!(saved.recipient(&context, &resumed).await.unwrap(), person);
    assert_eq!(
        agent_registry::load(&root, "researcher").unwrap().unwrap(),
        original
    );
    let other_chat = ChatId::create_for_contact(&context, other).await.unwrap();
    let second_args = parse(&["agent-connect", "grok", "--alias", "writer", "--to", "me"]).unwrap();
    let second = Agent::create(&root, &second_args, &context, other, other_chat)
        .await
        .unwrap();
    assert_eq!(second.id, 2);
    assert_eq!(
        agent_registry::resolve_selector(&root, "2").unwrap(),
        "writer"
    );
    assert!(agent_registry::resolve_selector(&root, "999").is_err());
    // Management works by stable numeric ID and preserves the binding when stopped.
    let mut runtime = crate::agents::register(&root, &args, chat.to_u32())
        .await
        .unwrap();
    let status = crate::agents::request(&root, "1", "status").await.unwrap();
    assert!(status.contains("\"alias\":\"researcher\""));
    assert!(status.contains("\"id\":1"));
    let stop = parse(&["agent-stop", "1", "--data-path", root.to_str().unwrap()]).unwrap();
    let task = tokio::spawn(async move { crate::agents::stop(&stop).await });
    tokio::time::timeout(std::time::Duration::from_secs(3), runtime.stop.changed())
        .await
        .unwrap()
        .unwrap();
    drop(runtime);
    task.await.unwrap().unwrap();
    assert_eq!(
        agent_registry::load(&root, "researcher").unwrap().unwrap(),
        original
    );
    assert_eq!(agent_registry::list(&root).unwrap().len(), 2);
    let list = parse(&["agent-list", "--data-path", root.to_str().unwrap()]).unwrap();
    crate::agents::list(&list).await.unwrap();
    drop(context);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_agent_starting_prompt_includes_model_without_losing_custom_instructions() {
    use crate::handlers::agent_connect::starting_prompt;
    assert_eq!(
        starting_prompt(Some("test-model"), None),
        "Model: test-model."
    );
    assert_eq!(
        starting_prompt(Some("test-model"), Some("Be precise.\nGrüße 🌍")),
        "Be precise.\nGrüße 🌍\n\nModel: test-model."
    );
    let default = starting_prompt(None, Some("Be helpful."));
    assert!(default.starts_with("Be helpful.\n\n"));
    assert!(default.contains("configured default"));
    assert!(default.contains("not supplied an explicit model ID"));
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn native_sessions_resume_exact_ids_and_retain_ids_after_failure() {
    use crate::agent_sessions::{self, Invocation};
    use std::os::unix::fs::PermissionsExt;
    let root = std::env::temp_dir().join(format!("sigma-provider-sessions-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let executable = root.join("mock agent");
    let write = |script: &str| {
        std::fs::write(&executable, script).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    };
    let codex = root.join("codex");
    let id = "11111111-1111-4111-8111-111111111111";
    write(&format!(
        r##"#!/bin/sh
[ "$1" = --no-daemon ] || exit 1
[ "$4" = exec ] || exit 4
[ "$6" = read-only ] || exit 6
case " $* " in *" --ephemeral "*|*" --last "*) exit 9 ;; esac
if [ "$7" = resume ]; then
 [ "$8" = '{id}' ] || exit 8
 [ "${{11}}" = --model ] || exit 11
 [ "${{12}}" = test-model ] || exit 12
 [ "${{13}}" = --config ] || exit 13
 case "${{14}}" in *"Model: test-model."*) ;; *) exit 14 ;; esac
else
 [ "$9" = --model ] || exit 9
 [ "${{10}}" = test-model ] || exit 10
 [ "${{11}}" = --config ] || exit 11
fi
payload=$(cat)
if [ "$7" = resume ]; then
 case "$payload" in *"<sigma_runtime_context>"*"Model: test-model."*"recall the word"*) ;; *) exit 15 ;; esac
fi
printf '%s\n' '{{"type":"thread.started","thread_id":"{id}"}}'
printf '%s\n' '{{"type":"item.completed","item":{{"type":"agent_message","text":"remembered"}}}}'
printf '%s\n' '{{"type":"turn.completed"}}'
"##
    ));
    let prompt = crate::handlers::agent_connect::starting_prompt(
        Some("test-model"),
        Some("Be helpful.\nGrüße 🌍"),
    );
    let invoke = Invocation {
        executable: &executable,
        provider: "codex",
        model: Some("test-model"),
        prompt: &prompt,
        seconds: 3,
        dir: &codex,
        environment: &[],
        workspace: None,
    };
    assert_eq!(invoke.run("remember a word").await.unwrap(), "remembered");
    assert_eq!(
        agent_sessions::load(&codex).unwrap().unwrap().id.as_deref(),
        Some(id)
    );
    assert_eq!(invoke.run("recall the word").await.unwrap(), "remembered");
    write(&format!(
        "#!/bin/sh\ncat >/dev/null\nprintf '%s\\n' '{{\"type\":\"thread.started\",\"thread_id\":\"{id}\"}}'\nexit 7\n"
    ));
    assert!(invoke.run("failure").await.is_err());
    assert_eq!(
        agent_sessions::load(&codex).unwrap().unwrap().id.as_deref(),
        Some(id)
    );
    write(
        "#!/bin/sh\ncat >/dev/null\nprintf '%s\\n' '{\"type\":\"thread.started\",\"thread_id\":\"22222222-2222-4222-8222-222222222222\"}'\n",
    );
    assert!(invoke.run("refuse a different thread").await.is_err());
    assert_eq!(
        agent_sessions::load(&codex).unwrap().unwrap().id.as_deref(),
        Some(id)
    );
    let pending = root.join("pending");
    write("#!/bin/sh\ncat >/dev/null\nexit 7\n");
    let pending_invoke = Invocation {
        dir: &pending,
        ..invoke
    };
    assert!(pending_invoke.run("interrupted creation").await.is_err());
    assert!(
        pending_invoke
            .run("do not create another thread")
            .await
            .unwrap_err()
            .to_string()
            .contains("interrupted")
    );
    let grok = root.join("grok");
    write(
        r##"#!/bin/sh
cat >/dev/null
previous=''
session=''
for arg in "$@"; do
 if [ "$previous" = --session-id ] || [ "$previous" = --resume ]; then session="$arg"; fi
 previous="$arg"
done
[ -n "$session" ] || exit 7
printf '{"text":"remembered","sessionId":"%s"}\n' "$session"
"##,
    );
    let invoke = Invocation {
        executable: &executable,
        provider: "grok",
        model: Some("test-model"),
        prompt: &prompt,
        seconds: 3,
        dir: &grok,
        environment: &[],
        workspace: None,
    };
    assert_eq!(invoke.run("remember a word").await.unwrap(), "remembered");
    let initial = agent_sessions::load(&grok).unwrap().unwrap();
    assert_eq!(invoke.run("recall the word").await.unwrap(), "remembered");
    assert_eq!(agent_sessions::load(&grok).unwrap().unwrap(), initial);
    write(
        "#!/bin/sh\ncat >/dev/null\nprintf '%s' '{\"text\":\"wrong\",\"sessionId\":\"22222222-2222-4222-8222-222222222222\"}'\n",
    );
    assert!(invoke.run("refuse a different session").await.is_err());
    assert_eq!(agent_sessions::load(&grok).unwrap().unwrap(), initial);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn session_bootstrap_uses_only_the_bound_chat_and_skips_notifications() {
    use deltachat::{contact::ContactId, message::Message};
    let root = std::env::temp_dir().join(format!("sigma-bootstrap-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let context = offline_identity(&root, "bootstrap-owner").await;
    let chat = ChatId::create_for_contact(&context, ContactId::SELF)
        .await
        .unwrap();
    let other_chat = chat::create_group(&context, "Unrelated").await.unwrap();
    let mut unrelated = Message::new_text("unrelated-secret".into());
    chat::send_msg(&context, other_chat, &mut unrelated)
        .await
        .unwrap();
    let mut notification =
        Message::new_text("Connected to Codex through Sigma. Worker started.".into());
    chat::send_msg(&context, chat, &mut notification)
        .await
        .unwrap();
    let mut first = Message::new_text("earlier answer with Unicode Grüße 🌍".into());
    chat::send_msg(&context, chat, &mut first).await.unwrap();
    let mut current = Message::new_text("what was earlier?".into());
    let id = chat::send_msg(&context, chat, &mut current).await.unwrap();
    let current = Message::load_from_db(&context, id).await.unwrap();
    let prompt = crate::agent_sessions::bootstrap(&context, &current)
        .await
        .unwrap();
    assert!(prompt.contains("earlier answer with Unicode Grüße 🌍"));
    assert!(prompt.contains("\"role\":\"assistant\""));
    assert!(!prompt.contains("unrelated-secret"));
    assert!(!prompt.contains("Worker started"));
    assert!(prompt.ends_with("Current user message:\nwhat was earlier?"));
    drop(context);
    std::fs::remove_dir_all(root).unwrap();
}

#[ignore = "explicit live provider test; uses installed CLI authentication and network"]
#[tokio::test(flavor = "multi_thread")]
async fn native_provider_sessions_live_memory() {
    let root = std::env::temp_dir().join(format!("sigma-live-session-{}", std::process::id()));
    for (provider, model) in [("codex", Some("gpt-6.1-sol")), ("grok", None)] {
        let executable = crate::handlers::agent_connect::executable(provider, None).unwrap();
        let dir = root.join(provider);
        let prompt = crate::handlers::agent_connect::starting_prompt(
            model,
            Some(
                "For this test, do not use tools. Continue the same conversation and remember what the user says.",
            ),
        );
        let invocation = crate::agent_sessions::Invocation {
            executable: &executable,
            provider,
            model,
            prompt: &prompt,
            seconds: 120,
            dir: &dir,
            environment: &[],
            workspace: None,
        };
        invocation
            .run("Remember the code word sigma-cobalt-47 for this conversation. Reply only saved.")
            .await
            .unwrap();
        let saved = crate::agent_sessions::load(&dir).unwrap().unwrap();
        let response = invocation
            .run("What code word did I ask you to remember? Reply only that code word.")
            .await
            .unwrap();
        assert!(
            response.contains("sigma-cobalt-47"),
            "{provider}: {response}"
        );
        assert_eq!(crate::agent_sessions::load(&dir).unwrap().unwrap(), saved);
        println!(
            "{provider}: remembered across turns using exact session {}",
            saved.id.unwrap()
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
async fn agent_action_call(
    server: &crate::agent_actions::Server,
    argv: &[&str],
    id: uuid::Uuid,
    token: Option<&str>,
) -> deltachat::webxdc::StatusUpdateItem {
    let env = |key| {
        server
            .environment
            .iter()
            .find(|(k, _)| k == key)
            .unwrap()
            .1
            .as_str()
    };
    let args = parse(argv).unwrap();
    let actions = crate::agent_actions::cli_actions(&args).unwrap();
    let raw = crate::agent_actions::request(
        std::path::Path::new(env("SIGMA_AGENT_SOCKET")),
        token.unwrap_or_else(|| env("SIGMA_AGENT_CAPABILITY")),
        &id.to_string(),
        &actions,
    )
    .await
    .unwrap();
    deltachat::webxdc::StatusUpdateItem {
        payload: raw.parse().unwrap(),
        ..Default::default()
    }
}

#[test]
fn agent_action_cli_rejects_scope_overrides_and_unknown_fields() {
    for argv in [
        vec!["agent-act", "messages"],
        vec!["agent-act", "messages", "20", "100"],
        vec!["agent-act", "send-message", "hello"],
        vec!["agent-act", "send-photo", "image.png", "--caption", "hello"],
        vec!["agent-act", "send-voice", "voice.ogg", "--duration", "3500"],
        vec![
            "agent-act",
            "send-video",
            "video.mp4",
            "--caption",
            "hello",
            "--duration",
            "1000",
        ],
        vec!["agent-act", "react", "10", "👍"],
        vec!["agent-act", "react", "10"],
        vec!["agent-act", "edit-message", "10", "fixed"],
        vec!["agent-act", "reply", "10", "hello"],
        vec!["agent-act", "set-photo", ""],
        vec!["agent-act", "set-name", "Codex"],
    ] {
        parse(&argv).unwrap().validate().unwrap();
    }
    for argv in [
        vec!["agent-act"],
        vec!["agent-act", "messages", "51"],
        vec!["agent-act", "unknown", "anything"],
        vec!["agent-act", "send-message", "hello", "--account", "other"],
        vec!["agent-act", "send-message", "hello", "--to", "other"],
        vec!["agent-act", "send-message", "hello", "--data-path", "other"],
        vec!["agent-act", "send-message", "hello", "--caption", "wrong"],
        vec!["agent-act", "send-photo", "photo.png", "--duration", "10"],
        vec![
            "agent-act",
            "send-message",
            "hello",
            "--request-id",
            "not-a-uuid",
        ],
        vec!["agent-act", "react", "0", "👍"],
        vec!["agent-act", "set-name", " "],
    ] {
        assert!(parse(&argv).and_then(|a| a.validate()).is_err(), "{argv:?}");
    }
    let file = std::env::temp_dir().join(format!("sigma-action-scope-{}.json", std::process::id()));
    std::fs::write(
        &file,
        r#"[{"action":"send-message","text":"hello","chat_id":10}]"#,
    )
    .unwrap();
    assert!(
        parse(&["agent-act", "batch", file.to_str().unwrap()])
            .unwrap()
            .validate()
            .is_err()
    );
    std::fs::remove_file(file).unwrap();
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn agent_actions_are_authenticated_scoped_and_idempotent_and_send_unquoted_media() {
    use deltachat::{
        config::Config,
        contact::ContactId,
        message::{Message, MsgId},
    };
    let root = socket_test_directory();
    std::fs::create_dir_all(&root).unwrap();
    let context = offline_identity(&root, "actions-owner").await;
    let chat = ChatId::create_for_contact(&context, ContactId::SELF)
        .await
        .unwrap();
    let other_chat = chat::create_group(&context, "Unrelated").await.unwrap();
    let mut other = Message::new_text("do not edit".into());
    let other_id = chat::send_msg(&context, other_chat, &mut other)
        .await
        .unwrap();
    let dir = root.join("agent");
    let server = crate::agent_actions::Server::start(&context, chat, &dir)
        .await
        .unwrap();
    let id = uuid::Uuid::new_v4();
    let argv = ["agent-act", "send-message", "standalone hello"];
    let rejected = agent_action_call(&server, &argv, id, Some("wrong-token")).await;
    assert_eq!(rejected.payload["ok"].as_bool(), Some(false));
    let first = agent_action_call(&server, &argv, id, None).await;
    assert_eq!(first.payload["ok"].as_bool(), Some(true));
    let message_id = MsgId::new(first.payload["results"][0]["message_id"].as_u64().unwrap() as u32);
    let sent = Message::load_from_db(&context, message_id).await.unwrap();
    assert_eq!(sent.get_text(), "standalone hello");
    assert!(sent.parent(&context).await.unwrap().is_none());
    let repeated = agent_action_call(&server, &argv, id, None).await;
    assert_eq!(first.payload, repeated.payload);
    let env = |key| {
        server
            .environment
            .iter()
            .find(|(k, _)| k == key)
            .unwrap()
            .1
            .as_str()
    };
    let actions = crate::agent_actions::cli_actions(&parse(&argv).unwrap()).unwrap();
    let signed = crate::agent_actions::signed_request(
        env("SIGMA_AGENT_CAPABILITY"),
        &uuid::Uuid::new_v4().to_string(),
        &actions,
    );
    assert!(!signed.contains(env("SIGMA_AGENT_CAPABILITY")));
    {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut stream = tokio::net::UnixStream::connect(env("SIGMA_AGENT_SOCKET"))
            .await
            .unwrap();
        stream
            .write_all(
                signed
                    .replace("standalone hello", "tampered content")
                    .as_bytes(),
            )
            .await
            .unwrap();
        stream.shutdown().await.unwrap();
        let mut result = String::new();
        stream.read_to_string(&mut result).await.unwrap();
        assert!(result.contains("unauthorized agent action"));
    }
    let mailbox_result = crate::agent_actions::request_mailbox(
        std::path::Path::new(env("SIGMA_AGENT_MAILBOX")),
        env("SIGMA_AGENT_CAPABILITY"),
        &id.to_string(),
        &actions,
    )
    .await
    .unwrap();
    let mailbox_value = deltachat::webxdc::StatusUpdateItem {
        payload: mailbox_result.parse().unwrap(),
        ..Default::default()
    };
    assert_eq!(mailbox_value.payload, first.payload);
    let changed = agent_action_call(
        &server,
        &["agent-act", "send-message", "different"],
        id,
        None,
    )
    .await;
    assert_eq!(changed.payload["ok"].as_bool(), Some(false));
    let invalid_edit = agent_action_call(
        &server,
        &[
            "agent-act",
            "edit-message",
            &other_id.to_u32().to_string(),
            "wrong chat",
        ],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(invalid_edit.payload["ok"].as_bool(), Some(false));
    assert_eq!(
        Message::load_from_db(&context, other_id)
            .await
            .unwrap()
            .get_text(),
        "do not edit"
    );
    let invalid_react = agent_action_call(
        &server,
        &["agent-act", "react", &other_id.to_u32().to_string(), "👍"],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(invalid_react.payload["ok"].as_bool(), Some(false));
    let invalid_reply = agent_action_call(
        &server,
        &[
            "agent-act",
            "reply",
            &other_id.to_u32().to_string(),
            "wrong chat",
        ],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(invalid_reply.payload["ok"].as_bool(), Some(false));
    let reply = agent_action_call(
        &server,
        &[
            "agent-act",
            "reply",
            &message_id.to_u32().to_string(),
            "explicit reply",
        ],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(reply.payload["ok"].as_bool(), Some(true));
    let reply_id = MsgId::new(reply.payload["results"][0]["message_id"].as_u64().unwrap() as u32);
    let reply_message = Message::load_from_db(&context, reply_id).await.unwrap();
    assert_eq!(reply_message.get_text(), "explicit reply");
    assert_eq!(
        reply_message
            .parent(&context)
            .await
            .unwrap()
            .unwrap()
            .get_id(),
        message_id
    );
    let history = agent_action_call(
        &server,
        &["agent-act", "messages", "50"],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(history.payload["ok"].as_bool(), Some(true));
    assert!(
        history.payload["results"][0]["messages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["message_id"].as_u64() == Some(message_id.to_u32().into()))
    );
    assert!(!history.payload.to_string().contains("do not edit"));
    let app = server.outbox.join("app.xdc");
    std::fs::write(
        &app,
        include_bytes!(concat!(env!("OUT_DIR"), "/minimal.xdc")),
    )
    .unwrap();
    let sent_app = agent_action_call(
        &server,
        &[
            "agent-act",
            "send-webxdc",
            "app.xdc",
            "--caption",
            "Try this",
        ],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(sent_app.payload["ok"].as_bool(), Some(true));
    let app_id = sent_app.payload["results"][0]["message_id"]
        .as_u64()
        .unwrap()
        .to_string();
    let updated = agent_action_call(
        &server,
        &[
            "agent-act",
            "webxdc-update",
            &app_id,
            "{\"payload\":{\"count\":1}}",
        ],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(updated.payload["ok"].as_bool(), Some(true));
    let updates = agent_action_call(
        &server,
        &["agent-act", "webxdc-updates", &app_id, "0"],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(updates.payload["ok"].as_bool(), Some(true));
    assert!(
        updates.payload["results"][0]["updates_json"]
            .as_str()
            .unwrap()
            .contains("count")
    );
    let wrong_type = agent_action_call(
        &server,
        &[
            "agent-act",
            "webxdc-update",
            &message_id.to_u32().to_string(),
            "{\"payload\":{}}",
        ],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(wrong_type.payload["ok"].as_bool(), Some(false));
    // An app in another chat is rejected before realtime/network side effects.
    let mut other_app = crate::handlers::send_webxdc::prepare(&context, &app, None)
        .await
        .unwrap();
    let other_app = chat::send_msg(&context, other_chat, &mut other_app)
        .await
        .unwrap();
    for action in ["webxdc-updates", "webxdc-update", "webxdc-realtime"] {
        let mut argv = vec!["agent-act", action];
        let id = other_app.to_u32().to_string();
        argv.push(&id);
        if action != "webxdc-updates" {
            argv.push("{\"payload\":{}}");
        }
        let denied = agent_action_call(&server, &argv, uuid::Uuid::new_v4(), None).await;
        assert_eq!(denied.payload["ok"].as_bool(), Some(false));
    }
    let image = server.outbox.join("image.png");
    std::fs::write(
        &image,
        include_bytes!(concat!(env!("OUT_DIR"), "/photo.png")),
    )
    .unwrap();
    std::fs::write(
        server.outbox.join("voice.wav"),
        include_bytes!(concat!(env!("OUT_DIR"), "/voice-tone.wav")),
    )
    .unwrap();
    std::fs::write(server.outbox.join("report.txt"), "report").unwrap();
    std::fs::write(server.outbox.join("video.mp4"), b"fixture video bytes").unwrap();
    for (action, path, typ) in [
        ("send-photo", "image.png", Viewtype::Image),
        ("send-voice", "voice.wav", Viewtype::Voice),
        ("send-file", "report.txt", Viewtype::File),
        ("send-video", "video.mp4", Viewtype::Video),
    ] {
        let result = agent_action_call(
            &server,
            &["agent-act", action, path, "--caption", "a caption"],
            uuid::Uuid::new_v4(),
            None,
        )
        .await;
        assert_eq!(
            result.payload["ok"].as_bool(),
            Some(true),
            "{}",
            result.payload
        );
        let mid = MsgId::new(result.payload["results"][0]["message_id"].as_u64().unwrap() as u32);
        let message = Message::load_from_db(&context, mid).await.unwrap();
        assert_eq!(message.get_viewtype(), typ);
        assert_eq!(message.get_text(), "a caption");
        assert!(message.parent(&context).await.unwrap().is_none());
    }
    let update = agent_action_call(
        &server,
        &[
            "agent-act",
            "edit-message",
            &message_id.to_u32().to_string(),
            "updated",
        ],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(
        update.payload["ok"].as_bool(),
        Some(true),
        "{}",
        update.payload
    );
    let reaction = agent_action_call(
        &server,
        &["agent-act", "react", &message_id.to_u32().to_string(), "👍"],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(reaction.payload["ok"].as_bool(), Some(true));
    let name = agent_action_call(
        &server,
        &["agent-act", "set-name", "My Agent"],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(name.payload["ok"].as_bool(), Some(true));
    assert_eq!(
        context
            .get_config(Config::Displayname)
            .await
            .unwrap()
            .as_deref(),
        Some("My Agent")
    );
    let photo = agent_action_call(
        &server,
        &["agent-act", "set-photo", "image.png"],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(photo.payload["ok"].as_bool(), Some(true));
    assert!(
        context
            .get_config(Config::Selfavatar)
            .await
            .unwrap()
            .is_some()
    );
    let batch = server.outbox.join("actions.json");
    std::fs::write(
        &batch,
        r#"[{"action":"send-message","text":"one"},{"action":"send-message","text":"two"}]"#,
    )
    .unwrap();
    let results = agent_action_call(
        &server,
        &["agent-act", "batch", batch.to_str().unwrap()],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(results.payload["results"].as_array().unwrap().len(), 2);
    let before = chat::get_chat_msgs(&context, chat).await.unwrap().len();
    std::fs::write(&batch, format!(r#"[{{"action":"send-message","text":"must-not-send"}},{{"action":"react","message_id":{},"text":"👍"}}]"#, other_id.to_u32())).unwrap();
    let preflight = agent_action_call(
        &server,
        &["agent-act", "batch", batch.to_str().unwrap()],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(preflight.payload["ok"].as_bool(), Some(false));
    assert_eq!(
        chat::get_chat_msgs(&context, chat).await.unwrap().len(),
        before
    );
    std::fs::write(root.join("private.txt"), "outside outbox").unwrap();
    std::os::unix::fs::symlink(root.join("private.txt"), server.outbox.join("link.txt")).unwrap();
    let escape = agent_action_call(
        &server,
        &["agent-act", "send-file", "link.txt"],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(escape.payload["ok"].as_bool(), Some(false));
    let second =
        crate::agent_actions::Server::start(&context, other_chat, &root.join("other-agent"))
            .await
            .unwrap();
    let old_token = server
        .environment
        .iter()
        .find(|(k, _)| k == "SIGMA_AGENT_CAPABILITY")
        .unwrap()
        .1
        .clone();
    let cross_token =
        agent_action_call(&second, &argv, uuid::Uuid::new_v4(), Some(&old_token)).await;
    assert_eq!(cross_token.payload["ok"].as_bool(), Some(false));
    drop(second);
    drop(server);
    let restarted = crate::agent_actions::Server::start(&context, chat, &dir)
        .await
        .unwrap();
    let old_capability =
        agent_action_call(&restarted, &argv, uuid::Uuid::new_v4(), Some(&old_token)).await;
    assert_eq!(old_capability.payload["ok"].as_bool(), Some(false));
    let cached_after_restart = agent_action_call(&restarted, &argv, id, None).await;
    assert_eq!(cached_after_restart.payload, first.payload);
    drop(restarted);
    drop(context);
    std::fs::remove_dir_all(root).unwrap();
}

#[ignore = "explicit live provider action test; uses installed CLI authentication and network"]
#[tokio::test(flavor = "multi_thread")]
async fn native_providers_can_use_scoped_action_tools() {
    use deltachat::{chat::ChatItem, contact::ContactId, message::Message};
    let root = std::env::temp_dir().join(format!("sigma-live-actions-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let context = offline_identity(&root, "live-actions").await;
    let chat = ChatId::create_for_contact(&context, ContactId::SELF)
        .await
        .unwrap();
    let binary = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/sigma");
    assert!(binary.is_file(), "run cargo build before this live test");
    for (provider, model) in [("codex", Some("gpt-6.1-sol")), ("grok", None)] {
        let executable = crate::handlers::agent_connect::executable(provider, None).unwrap();
        let dir = root.join(provider);
        let mut server = crate::agent_actions::Server::start(&context, chat, &dir)
            .await
            .unwrap();
        server
            .environment
            .iter_mut()
            .find(|(key, _)| key == "SIGMA_BIN")
            .unwrap()
            .1 = binary.to_string_lossy().into_owned();
        server.environment.push((
            "SIGMA_AGENT_SANDBOX_ROOT".into(),
            root.to_string_lossy().into_owned(),
        ));
        server.environment.push((
            "SIGMA_AGENT_STATE_DIR".into(),
            dir.to_string_lossy().into_owned(),
        ));
        let prompt = format!(
            "{}\n\n{}",
            crate::handlers::agent_connect::starting_prompt(
                model,
                Some("Use the provided Sigma tools to fulfill the chat request.")
            ),
            server.instructions()
        );
        let original_environment = server
            .environment
            .iter()
            .filter(|(key, _)| {
                !matches!(
                    key.as_str(),
                    "SIGMA_AGENT_SANDBOX_ROOT" | "SIGMA_AGENT_STATE_DIR"
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        crate::agent_sessions::Invocation {
            executable: &executable,
            provider,
            model,
            prompt: &prompt,
            seconds: 180,
            dir: &dir,
            environment: &original_environment,
            workspace: Some(&server.outbox),
        }
        .run("Remember the phrase sigma isolation migration. Reply briefly that you are ready.")
        .await
        .unwrap();
        let before_migration = crate::agent_sessions::load(&dir).unwrap();
        let invocation = crate::agent_sessions::Invocation {
            executable: &executable,
            provider,
            model,
            prompt: &prompt,
            seconds: 180,
            dir: &dir,
            environment: &server.environment,
            workspace: Some(&server.outbox),
        };
        let text = format!("live-action-check-{provider}");
        let response = invocation.run(&format!("Use the actual Sigma send-message command to send exactly {text} as an unquoted standalone message. Do not print environment values. After the tool succeeds, return exactly [SIGMA_NO_REPLY].")).await.unwrap();
        assert!(
            response.trim().ends_with(crate::agent_actions::NO_REPLY),
            "{provider}: {response}"
        );
        assert_eq!(crate::agent_sessions::load(&dir).unwrap(), before_migration);
        let mut found = false;
        for item in chat::get_chat_msgs(&context, chat).await.unwrap() {
            if let ChatItem::Message { msg_id } = item {
                let message = Message::load_from_db(&context, msg_id).await.unwrap();
                if message.get_text() == text {
                    assert!(message.parent(&context).await.unwrap().is_none());
                    found = true;
                }
            }
        }
        assert!(found, "{provider} did not invoke the action service");
        println!(
            "{provider}: authenticated action queued an unquoted message in the fixed offline chat"
        );
        drop(server);
    }
    drop(context);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn agent_permissions_cli_validates_operations_and_scope() {
    for command in [
        "/access",
        "/access remove 10",
        "/approve",
        "/assign",
        "/revoke",
    ] {
        assert!(crate::agent_guard::is_command(command));
    }
    for text in ["/accessibility", "please /access", "quoted: /approve"] {
        assert!(!crate::agent_guard::is_command(text));
    }

    for argv in [
        vec!["agent-permissions", "codex", "list"],
        vec!["agent-permissions", "1", "approve", "12", "15"],
        vec![
            "agent-permissions",
            "codex",
            "assign",
            "12",
            "15",
            "--account",
            "codex-new",
        ],
        vec!["agent-permissions", "codex", "revoke", "12", "15"],
        vec!["agent-act", "send-webxdc", "app.xdc", "--caption", "App"],
        vec!["agent-act", "webxdc-update", "12", "{\"payload\":{}}"],
        vec!["agent-act", "webxdc-updates", "12", "0"],
        vec!["agent-act", "webxdc-realtime", "12", "hello"],
    ] {
        parse(&argv).unwrap().validate().unwrap();
    }
    for argv in [
        vec!["agent-permissions"],
        vec!["agent-permissions", "codex", "approve"],
        vec!["agent-permissions", "codex", "approve", "0", "12"],
        vec!["agent-permissions", "codex", "replace-owner", "12", "15"],
        vec!["agent-act", "approve", "12"],
        vec!["agent-act", "webxdc-update", "12"],
    ] {
        assert!(
            parse(&argv).and_then(|args| args.validate()).is_err(),
            "{argv:?}"
        );
    }
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn agent_guard_scopes_approvals_and_revokes_live_action_credentials() {
    use deltachat::{
        contact::{self, ContactId},
        message::Message,
    };
    let root = socket_test_directory();
    std::fs::create_dir_all(&root).unwrap();
    let context = offline_identity(&root, "guard-bot").await;
    let owner_context = offline_identity(&root, "guard-owner").await;
    let guest_context = offline_identity(&root, "guard-guest").await;
    let owner = contact::import_vcard(
        &context,
        &contact::make_vcard(&owner_context, &[ContactId::SELF])
            .await
            .unwrap(),
    )
    .await
    .unwrap()[0];
    let guest = contact::import_vcard(
        &context,
        &contact::make_vcard(&guest_context, &[ContactId::SELF])
            .await
            .unwrap(),
    )
    .await
    .unwrap()[0];
    let direct = ChatId::create_for_contact(&context, owner).await.unwrap();
    let group = chat::create_group(&context, "Owner group").await.unwrap();
    let other = chat::create_group(&context, "Other group").await.unwrap();
    for chat in [group, other] {
        crate::handlers::add_to_group::add_member(&context, chat, owner)
            .await
            .unwrap();
        crate::handlers::add_to_group::add_member(&context, chat, guest)
            .await
            .unwrap();
    }
    let dir = root.join("agent");
    std::fs::create_dir_all(&dir).unwrap();
    assert!(
        crate::agent_guard::sender_allowed(&context, &dir, owner, direct, direct, owner)
            .await
            .unwrap()
    );
    assert!(
        crate::agent_guard::sender_allowed(&context, &dir, owner, direct, group, owner)
            .await
            .unwrap()
    );
    assert!(
        !crate::agent_guard::sender_allowed(&context, &dir, owner, direct, group, guest)
            .await
            .unwrap()
    );
    let mut scope = crate::agent_guard::Scope {
        dir: dir.clone(),
        owner,
        owner_chat: direct,
        chat: group,
        sender: owner,
    };
    assert!(
        crate::agent_guard::quoted_command(&context, &scope, "/approve", None)
            .await
            .unwrap()
            .unwrap()
            .contains("Reply")
    );
    assert!(
        crate::agent_guard::quoted_command(
            &context,
            &scope,
            "/approve",
            Some((other, guest, false))
        )
        .await
        .unwrap()
        .unwrap()
        .contains("Choose")
    );
    scope.sender = guest;
    assert!(
        crate::agent_guard::quoted_command(
            &context,
            &scope,
            "/approve",
            Some((group, guest, false))
        )
        .await
        .unwrap()
        .is_none()
    );
    assert!(
        !crate::agent_guard::sender_allowed(&context, &dir, owner, direct, group, guest)
            .await
            .unwrap()
    );
    scope.sender = owner;
    assert!(
        crate::agent_guard::quoted_command(
            &context,
            &scope,
            "/assign",
            Some((group, guest, false))
        )
        .await
        .unwrap()
        .unwrap()
        .contains("Approved")
    );
    assert!(
        crate::agent_guard::sender_allowed(&context, &dir, owner, direct, group, guest)
            .await
            .unwrap()
    );
    crate::agent_guard::quoted_command(&context, &scope, "/revoke", Some((group, guest, false)))
        .await
        .unwrap();
    // Owner-only chat access lists do not disclose other groups in a group.
    for chat in [group, other] {
        crate::agent_guard::change(&context, &dir, owner, direct, "approve", chat, guest)
            .await
            .unwrap();
    }
    let list = crate::agent_guard::quoted_command(&context, &scope, "/access", None)
        .await
        .unwrap()
        .unwrap();
    assert!(list.contains("guard-guest"));
    assert!(list.contains(&format!("chat {}: active", group.to_u32())));
    assert!(!list.contains(&format!("chat {}:", other.to_u32())));
    scope.sender = guest;
    for command in [
        "/access".to_owned(),
        format!("/access remove {}", guest.to_u32()),
    ] {
        assert!(
            crate::agent_guard::quoted_command(&context, &scope, &command, None)
                .await
                .unwrap()
                .is_none()
        );
    }
    assert!(
        crate::agent_guard::sender_allowed(&context, &dir, owner, direct, group, guest)
            .await
            .unwrap()
    );
    scope.sender = owner;
    let remove = format!("/access remove {}", guest.to_u32());
    assert!(
        crate::agent_guard::quoted_command(&context, &scope, &remove, None)
            .await
            .unwrap()
            .unwrap()
            .contains("Removed 1")
    );
    assert!(
        !crate::agent_guard::sender_allowed(&context, &dir, owner, direct, group, guest)
            .await
            .unwrap()
    );
    assert!(
        crate::agent_guard::sender_allowed(&context, &dir, owner, direct, other, guest)
            .await
            .unwrap()
    );
    scope.chat = direct;
    let list = crate::agent_guard::quoted_command(&context, &scope, "/access", None)
        .await
        .unwrap()
        .unwrap();
    assert!(list.contains(&format!("chat {}:", other.to_u32())));
    assert!(
        crate::agent_guard::quoted_command(
            &context,
            &scope,
            &format!("/access remove {}", owner.to_u32()),
            None
        )
        .await
        .unwrap()
        .unwrap()
        .contains("cannot be removed")
    );
    assert!(
        crate::agent_guard::quoted_command(&context, &scope, "/access remove invalid", None)
            .await
            .unwrap()
            .unwrap()
            .contains("Use /access")
    );
    assert!(
        crate::agent_guard::quoted_command(&context, &scope, "/access 0", None)
            .await
            .unwrap()
            .unwrap()
            .contains("Use /access")
    );
    assert!(
        crate::agent_guard::quoted_command(&context, &scope, "/access 99", None)
            .await
            .unwrap()
            .unwrap()
            .contains("Use /access 1")
    );
    crate::agent_guard::change(&context, &dir, owner, direct, "approve", group, guest)
        .await
        .unwrap();
    assert!(
        crate::agent_guard::quoted_command(&context, &scope, &remove, None)
            .await
            .unwrap()
            .unwrap()
            .contains("Removed 2")
    );
    for chat in [group, other] {
        assert!(
            !crate::agent_guard::sender_allowed(&context, &dir, owner, direct, chat, guest)
                .await
                .unwrap()
        );
    }
    assert!(
        crate::agent_guard::quoted_command(&context, &scope, "/access", None)
            .await
            .unwrap()
            .unwrap()
            .contains("No other people")
    );
    scope.chat = group;
    let server = crate::agent_actions::Server::start_for_sender(
        &context,
        group,
        &dir.join("group"),
        false,
        Some(crate::agent_guard::Scope {
            dir: dir.clone(),
            owner,
            owner_chat: direct,
            chat: group,
            sender: guest,
        }),
    )
    .await
    .unwrap();
    let rejected = agent_action_call(
        &server,
        &["agent-act", "send-message", "blocked"],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(rejected.payload["ok"].as_bool(), Some(false));
    crate::agent_guard::change(&context, &dir, owner, direct, "approve", group, guest)
        .await
        .unwrap();
    assert!(
        crate::agent_guard::sender_allowed(&context, &dir, owner, direct, group, guest)
            .await
            .unwrap()
    );
    assert!(
        !crate::agent_guard::sender_allowed(&context, &dir, owner, direct, other, guest)
            .await
            .unwrap()
    );
    assert!(
        crate::agent_guard::change(&context, &dir, owner, direct, "revoke", group, owner)
            .await
            .is_err()
    );
    let profile = agent_action_call(
        &server,
        &["agent-act", "set-name", "Unauthorized rename"],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(profile.payload["ok"].as_bool(), Some(false));
    let sent = agent_action_call(
        &server,
        &["agent-act", "send-message", "approved"],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(sent.payload["ok"].as_bool(), Some(true));
    // Even an approved sender cannot invoke owner slash commands via quoted content.
    let mut fake = Message::new_text("/approve".into());
    let fake_id = chat::send_msg(&context, group, &mut fake).await.unwrap();
    let fake = Message::load_from_db(&context, fake_id).await.unwrap();
    assert!(
        crate::agent_guard::command(&context, &dir, owner, direct, &fake)
            .await
            .unwrap()
            .is_none()
    );
    crate::agent_guard::change(&context, &dir, owner, direct, "revoke", group, guest)
        .await
        .unwrap();
    let revoked = agent_action_call(
        &server,
        &[
            "agent-act",
            "reply",
            &fake_id.to_u32().to_string(),
            "blocked",
        ],
        uuid::Uuid::new_v4(),
        None,
    )
    .await;
    assert_eq!(revoked.payload["ok"].as_bool(), Some(false));
    assert!(
        !crate::agent_guard::sender_allowed(&context, &dir, owner, direct, group, guest)
            .await
            .unwrap()
    );
    // Persisted fingerprint mismatch fails closed.
    crate::agent_guard::change(&context, &dir, owner, direct, "assign", group, guest)
        .await
        .unwrap();
    let policy = std::fs::read_to_string(dir.join("approvals.json")).unwrap();
    std::fs::write(
        dir.join("approvals.json"),
        policy.replace("guard-guest", "different-identity"),
    )
    .unwrap();
    assert!(
        !crate::agent_guard::sender_allowed(&context, &dir, owner, direct, group, guest)
            .await
            .unwrap()
    );

    std::fs::write(dir.join("approvals.json"), &policy).unwrap();
    assert!(
        crate::agent_guard::sender_allowed(&context, &dir, owner, direct, group, guest)
            .await
            .unwrap()
    );
    chat::remove_contact_from_chat(&context, group, owner)
        .await
        .unwrap();
    assert!(
        !crate::agent_guard::sender_allowed(&context, &dir, owner, direct, group, guest)
            .await
            .unwrap()
    );
    assert!(
        !crate::agent_guard::sender_allowed(&context, &dir, owner, direct, group, owner)
            .await
            .unwrap()
    );
    drop(server);
    drop(context);
    drop(owner_context);
    drop(guest_context);
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn agent_permission_cli_works_online_under_lock_and_offline() {
    use deltachat::contact::{self, ContactId};
    let root = socket_test_directory();
    std::fs::create_dir_all(&root).unwrap();
    let context = offline_identity(&root, "account").await;
    let person = offline_identity(&root, "permission-owner").await;
    let guest_context = offline_identity(&root, "permission-guest").await;
    let owner = contact::import_vcard(
        &context,
        &contact::make_vcard(&person, &[ContactId::SELF])
            .await
            .unwrap(),
    )
    .await
    .unwrap()[0];
    let guest = contact::import_vcard(
        &context,
        &contact::make_vcard(&guest_context, &[ContactId::SELF])
            .await
            .unwrap(),
    )
    .await
    .unwrap()[0];
    let direct = ChatId::create_for_contact(&context, owner).await.unwrap();
    let group = chat::create_group(&context, "CLI group").await.unwrap();
    for contact in [owner, guest] {
        crate::handlers::add_to_group::add_member(&context, group, contact)
            .await
            .unwrap();
    }
    let args = parse(&[
        "agent-connect",
        "codex",
        "--to",
        "me",
        "--data-path",
        root.to_str().unwrap(),
    ])
    .unwrap();
    crate::agent_registry::Agent::create(&root, &args, &context, owner, direct)
        .await
        .unwrap();
    let dir = root.join("agents/codex");
    let lock = crate::account::AccountLock::acquire(root.join("sigma.lock")).unwrap();
    let runtime =
        crate::agents::register_with_context(&root, &args, direct.to_u32(), Some(context.clone()))
            .await
            .unwrap();
    let chat_id = group.to_u32().to_string();
    let contact_id = guest.to_u32().to_string();
    for operation in ["approve", "revoke", "assign"] {
        let args = parse(&[
            "agent-permissions",
            "codex",
            operation,
            &chat_id,
            &contact_id,
            "--data-path",
            root.to_str().unwrap(),
        ])
        .unwrap();
        args.validate().unwrap();
        crate::handlers::agent_permissions::handle(&args)
            .await
            .unwrap();
        assert_eq!(
            crate::agent_guard::sender_allowed(&context, &dir, owner, direct, group, guest)
                .await
                .unwrap(),
            operation != "revoke"
        );
    }
    crate::agent_models::initialize(&dir, Some("test-model")).unwrap();
    for args in [
        vec!["agent-model", "codex"],
        vec!["agent-model", "codex", "default"],
    ] {
        let mut argv = args;
        argv.extend(["--data-path", root.to_str().unwrap()]);
        crate::handlers::agent_model::handle(&parse(&argv).unwrap())
            .await
            .unwrap();
    }
    assert_eq!(crate::agent_models::selected(&dir, None).unwrap(), None);
    let mut scope = crate::agent_guard::Scope {
        dir: dir.clone(),
        owner,
        owner_chat: direct,
        chat: group,
        sender: guest,
    };
    for command in ["/model", "/model default", "/models"] {
        assert!(
            crate::agent_guard::quoted_command(&context, &scope, command, None)
                .await
                .unwrap()
                .is_none()
        );
    }
    scope.sender = owner;
    assert!(
        crate::agent_guard::quoted_command(&context, &scope, "/model", None)
            .await
            .unwrap()
            .unwrap()
            .contains("configured default")
    );
    assert!(
        crate::agent_guard::quoted_command(&context, &scope, "/model default", None)
            .await
            .unwrap()
            .unwrap()
            .contains("Saved model")
    );
    drop(runtime);
    drop(lock);
    drop(context);
    tokio::task::yield_now().await;
    let args = parse(&[
        "agent-permissions",
        "1",
        "revoke",
        &chat_id,
        &contact_id,
        "--data-path",
        root.to_str().unwrap(),
    ])
    .unwrap();
    crate::handlers::agent_permissions::handle(&args)
        .await
        .unwrap();
    crate::handlers::agent_model::handle(
        &parse(&[
            "agent-model",
            "1",
            "default",
            "--data-path",
            root.to_str().unwrap(),
        ])
        .unwrap(),
    )
    .await
    .unwrap();
    let context = ContextBuilder::new(root.join("account.db"))
        .open()
        .await
        .unwrap();
    assert!(
        !crate::agent_guard::sender_allowed(&context, &dir, owner, direct, group, guest)
            .await
            .unwrap()
    );
    drop(context);
    drop(person);
    drop(guest_context);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn agent_model_commands_and_provider_catalogs_are_validated() {
    for argv in [
        vec!["agent-models", "codex"],
        vec!["agent-model", "1"],
        vec![
            "agent-model",
            "grok",
            "grok-example",
            "--account",
            "grok-new",
        ],
        vec!["agent-model", "codex", "default"],
    ] {
        parse(&argv).unwrap().validate().unwrap();
    }
    for argv in [
        vec!["agent-models"],
        vec!["agent-model"],
        vec!["agent-model", "codex", "bad model"],
        vec!["agent-model", "codex", "test", "extra"],
        vec!["agent-act", "model", "test"],
    ] {
        assert!(parse(&argv).and_then(|args| args.validate()).is_err());
    }
    for command in ["/model", "/model example", "/models"] {
        assert!(crate::agent_guard::is_command(command));
    }
    let codex = crate::agent_models::parse_catalog("codex", r#"{"fetched_at":"today","models":[{"slug":"model-b","display_name":"B","visibility":"list"},{"slug":"internal","visibility":"hide"},{"slug":"model-a","display_name":"A","api_key":"SHOULD_NOT_LEAK"}]}"#).unwrap();
    assert_eq!(
        codex
            .models
            .iter()
            .map(|m| m.id.as_str())
            .collect::<Vec<_>>(),
        ["model-a", "model-b"]
    );
    assert_eq!(codex.fetched_at, "today");
    let grok = crate::agent_models::parse_catalog("grok", r#"{"models":{"grok-test":{"info":{"name":"Grok Test"},"api_key":"SHOULD_NOT_LEAK","api_base_url":"private-url"}}}"#).unwrap();
    assert_eq!(grok.models[0].id, "grok-test");
    assert_eq!(grok.models[0].name, "Grok Test");
    assert!(!format!("{:?}", grok.models).contains("SHOULD_NOT_LEAK"));
    assert!(crate::agent_models::parse_catalog("unknown", "{}").is_err());
    assert!(crate::agent_models::parse_catalog("codex", r#"{"models":[]}"#).is_err());
}

#[test]
fn agent_model_selection_persists_without_replacing_sessions() {
    let root = std::env::temp_dir().join(format!("sigma-model-setting-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(root.join("chat-123")).unwrap();
    let session = r#"{"provider":"codex","session_id":"12345678-1234-4234-8234-123456789abc"}"#;
    for dir in [&root, &root.join("chat-123")] {
        std::fs::write(dir.join("session.json"), session).unwrap();
    }
    crate::agent_models::initialize(&root, Some("initial-model")).unwrap();
    assert_eq!(
        crate::agent_models::selected(&root, None)
            .unwrap()
            .as_deref(),
        Some("initial-model")
    );
    let catalog =
        crate::agent_models::parse_catalog("codex", r#"{"models":[{"slug":"new-model"}]}"#)
            .unwrap();
    assert!(crate::agent_models::set_from_catalog(&root, "nonexistent", &catalog).is_err());
    assert_eq!(
        crate::agent_models::selected(&root, None)
            .unwrap()
            .as_deref(),
        Some("initial-model")
    );
    crate::agent_models::set_from_catalog(&root, "new-model", &catalog).unwrap();
    crate::agent_models::initialize(&root, Some("old-worker-flag")).unwrap();
    assert_eq!(
        crate::agent_models::selected(&root, Some("old-worker-flag"))
            .unwrap()
            .as_deref(),
        Some("new-model")
    );
    assert!(crate::agent_models::control(&root, "codex", false, &["set", "default"]).is_err());
    assert!(crate::agent_models::control(&root, "adapter", true, &["get"]).is_err());
    crate::agent_models::control(&root, "codex", true, &["set", "default"]).unwrap();
    assert_eq!(
        crate::agent_models::selected(&root, Some("old-worker-flag")).unwrap(),
        None
    );
    for dir in [&root, &root.join("chat-123")] {
        assert_eq!(
            std::fs::read_to_string(dir.join("session.json")).unwrap(),
            session
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn incoming_voice_is_staged_for_agent_transcription_and_unsupported_reacts() {
    use deltachat::{contact::ContactId, message::Message, reaction};
    use std::os::unix::fs::PermissionsExt;
    let root = std::env::temp_dir().join(format!("sigma-incoming-voice-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let context = offline_identity(&root, "voice-agent").await;
    let chat = ChatId::create_for_contact(&context, ContactId::SELF)
        .await
        .unwrap();
    let audio = root.join("recording.wav");
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/voice-tone.wav"));
    std::fs::write(&audio, bytes).unwrap();
    let mut voice =
        crate::handlers::send_voice::prepare(&context, &audio, None, Some("1000")).unwrap();
    let voice_id = chat::send_msg(&context, chat, &mut voice).await.unwrap();
    let voice = Message::load_from_db(&context, voice_id).await.unwrap();
    assert!(voice.get_text().is_empty());
    assert!(crate::agent_voice::is_audio(&voice));
    let server = crate::agent_actions::Server::start(&context, chat, &root.join("agent"))
        .await
        .unwrap();
    let settings = crate::handlers::agent_connect::ReplySettings {
        native: None,
        model: None,
        system_prompt: None,
        session_dir: None,
        actions: Some(&server),
        import_history: false,
    };
    let supported = root.join("supported-adapter");
    // Simulate an adapter with its own transcription capability. The fixture is a tone,
    // so this tests transport/agent protocol rather than speech recognition accuracy.
    std::fs::write(
        &supported,
        r#"#!/bin/sh
cat > "$SIGMA_AGENT_OUTBOX/received-input.txt"
test -s "$SIGMA_INCOMING_VOICE_PATH" || exit 2
printf '%s' "$SIGMA_INCOMING_VOICE_PATH" > "$SIGMA_AGENT_OUTBOX/received-path.txt"
printf '%s' "$SIGMA_INCOMING_VOICE_DURATION_MS" > "$SIGMA_AGENT_OUTBOX/received-duration.txt"
printf '%s' 'Mock adapter handled its transcribed request.'
"#,
    )
    .unwrap();
    std::fs::set_permissions(&supported, std::fs::Permissions::from_mode(0o700)).unwrap();
    let reply_id =
        crate::handlers::agent_connect::reply(&context, &supported, 5, &settings, &voice)
            .await
            .unwrap()
            .unwrap();
    let reply = Message::load_from_db(&context, reply_id).await.unwrap();
    assert_eq!(
        reply.get_text(),
        "Mock adapter handled its transcribed request."
    );
    assert_eq!(
        reply.parent(&context).await.unwrap().unwrap().get_id(),
        voice_id
    );
    let staged = std::fs::read_to_string(server.outbox.join("received-path.txt")).unwrap();
    assert!(std::path::Path::new(&staged).starts_with(&server.outbox));
    assert_eq!(std::fs::read(staged).unwrap(), bytes);
    assert_eq!(
        std::fs::read_to_string(server.outbox.join("received-duration.txt")).unwrap(),
        "1000"
    );
    let prompt = std::fs::read_to_string(server.outbox.join("received-input.txt")).unwrap();
    assert!(prompt.contains("transcribe this recording yourself"));
    assert!(prompt.contains("caption above is not a transcript"));
    assert!(prompt.contains(crate::agent_voice::UNSUPPORTED));
    assert!(
        server
            .instructions()
            .contains(crate::agent_voice::UNSUPPORTED)
    );
    let unsupported = root.join("unsupported-adapter");
    std::fs::write(
        &unsupported,
        "#!/bin/sh\ncat >/dev/null\nprintf '%s' 'Internal commentary [SIGMA_VOICE_UNSUPPORTED]'\n",
    )
    .unwrap();
    std::fs::set_permissions(&unsupported, std::fs::Permissions::from_mode(0o700)).unwrap();
    let response =
        crate::handlers::agent_connect::reply(&context, &unsupported, 5, &settings, &voice)
            .await
            .unwrap();
    assert!(response.is_none());
    let reactions = reaction::get_msg_reactions(&context, voice_id)
        .await
        .unwrap();
    assert!(
        reactions
            .frequencies
            .iter()
            .any(|entry| entry.reaction.as_str() == "❌" && entry.is_from_self)
    );
    let mut normal = Message::new_text("Ordinary text".into());
    let normal_id = chat::send_msg(&context, chat, &mut normal).await.unwrap();
    let normal = Message::load_from_db(&context, normal_id).await.unwrap();
    assert!(!crate::agent_voice::is_audio(&normal));
    assert!(
        reaction::get_msg_reactions(&context, normal_id)
            .await
            .unwrap()
            .frequencies
            .is_empty()
    );
    std::fs::remove_file(voice.get_file(&context).unwrap()).unwrap();
    assert!(
        crate::handlers::agent_connect::reply(
            &context,
            &root.join("must-not-run"),
            5,
            &settings,
            &voice
        )
        .await
        .unwrap()
        .is_none()
    );
    drop(server);
    drop(context);
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(target_os = "linux")]
#[tokio::test(flavor = "multi_thread")]
async fn isolated_agent_cannot_read_or_modify_account_or_host_files() {
    if !std::path::Path::new("/usr/bin/bwrap").is_file() {
        return;
    }
    let root = std::env::temp_dir().join(format!("sigma-isolation-{}", uuid::Uuid::new_v4()));
    let account = root.join("account");
    let state = account.join("agents/test");
    let outbox = state.join("outbox");
    std::fs::create_dir_all(&outbox).unwrap();
    std::fs::write(account.join("account.db"), "trusted database").unwrap();
    std::fs::write(state.join("master.json"), "trusted master").unwrap();
    let host_file =
        std::path::PathBuf::from("/var/tmp").join(format!("sigma-host-{}", uuid::Uuid::new_v4()));
    std::fs::write(&host_file, "trusted host file").unwrap();
    let env = vec![
        (
            "SIGMA_AGENT_SANDBOX_ROOT".into(),
            account.to_string_lossy().into_owned(),
        ),
        (
            "SIGMA_AGENT_STATE_DIR".into(),
            state.to_string_lossy().into_owned(),
        ),
        (
            "SIGMA_AGENT_OUTBOX".into(),
            outbox.to_string_lossy().into_owned(),
        ),
        (
            "SIGMA_TEST_HOST_PATH".into(),
            host_file.to_string_lossy().into_owned(),
        ),
    ];
    let script = r#"import os
from pathlib import Path
account=Path(os.environ['SIGMA_AGENT_SANDBOX_ROOT'])
outbox=Path(os.environ['SIGMA_AGENT_OUTBOX'])
assert not (account/'account.db').exists()
assert not (outbox.parent/'master.json').exists()
# Fake files in the private empty mount cannot affect the host.
try: (account/'account.db').write_text('attack')
except OSError: pass
try: (outbox.parent/'master.json').write_text('attack')
except OSError: pass
try: Path(os.environ['SIGMA_TEST_HOST_PATH']).write_text('attack')
except OSError: pass
else: raise AssertionError('host path was writable')
(outbox/'generated.txt').write_text('permitted asset')
print('isolated')
"#;
    let response = crate::automation::run_process_env(
        std::path::Path::new("/usr/bin/python3"),
        &["-".into()],
        script,
        10,
        &env,
    )
    .await
    .unwrap();
    assert_eq!(response.trim(), "isolated");
    assert_eq!(
        std::fs::read_to_string(account.join("account.db")).unwrap(),
        "trusted database"
    );
    assert_eq!(
        std::fs::read_to_string(state.join("master.json")).unwrap(),
        "trusted master"
    );
    assert_eq!(
        std::fs::read_to_string(&host_file).unwrap(),
        "trusted host file"
    );
    assert_eq!(
        std::fs::read_to_string(outbox.join("generated.txt")).unwrap(),
        "permitted asset"
    );
    std::fs::remove_file(host_file).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn bot_master_transfer_is_exclusive_and_invalidates_old_scopes() {
    use deltachat::contact::{self, ContactId};
    let root = socket_test_directory();
    std::fs::create_dir_all(&root).unwrap();
    let context = offline_identity(&root, "master-bot").await;
    let first = offline_identity(&root, "first-master").await;
    let second = offline_identity(&root, "next-master").await;
    let old = contact::import_vcard(
        &context,
        &contact::make_vcard(&first, &[ContactId::SELF])
            .await
            .unwrap(),
    )
    .await
    .unwrap()[0];
    let next = contact::import_vcard(
        &context,
        &contact::make_vcard(&second, &[ContactId::SELF])
            .await
            .unwrap(),
    )
    .await
    .unwrap()[0];
    let direct = ChatId::create_for_contact(&context, old).await.unwrap();
    let group = chat::create_group(&context, "Master group").await.unwrap();
    for person in [old, next] {
        crate::handlers::add_to_group::add_member(&context, group, person)
            .await
            .unwrap();
    }
    let args = parse(&["agent-connect", "codex", "--to", "me"]).unwrap();
    let agent = crate::agent_registry::Agent::create(&root, &args, &context, old, direct)
        .await
        .unwrap();
    let dir = root.join("agents/codex");
    crate::agent_guard::change(&context, &dir, old, direct, "approve", group, next)
        .await
        .unwrap();
    let scope = crate::agent_guard::Scope {
        dir: dir.clone(),
        owner: old,
        owner_chat: direct,
        chat: direct,
        sender: old,
    };
    assert!(scope.check(&context).await.is_ok());
    assert!(
        crate::agent_master::transfer(&context, &dir, &agent, Some(next), next)
            .await
            .is_err()
    );
    let response = crate::agent_guard::quoted_command(
        &context,
        &crate::agent_guard::Scope {
            chat: group,
            ..scope.clone()
        },
        "/master",
        Some((group, next, false)),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(response.contains("transferred"));
    let master = crate::agent_master::current(&context, &dir, &agent)
        .await
        .unwrap();
    assert_eq!(master.contact, next);
    assert_ne!(master.contact, old);
    assert!(scope.check(&context).await.is_err());
    assert!(
        !crate::agent_guard::sender_allowed(&context, &dir, old, direct, direct, old)
            .await
            .unwrap()
    );
    assert!(
        crate::agent_guard::sender_allowed(
            &context,
            &dir,
            master.contact,
            master.chat,
            master.chat,
            next
        )
        .await
        .unwrap()
    );
    assert!(
        !crate::agent_guard::sender_allowed(
            &context,
            &dir,
            master.contact,
            master.chat,
            group,
            old
        )
        .await
        .unwrap()
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("approvals.json")).unwrap(),
        "{}"
    );
    assert!(
        crate::agent_guard::quoted_command(&context, &scope, "/master", Some((direct, old, false)))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        crate::agent_master::transfer(&context, &dir, &agent, Some(old), old)
            .await
            .is_err()
    );
    let restored =
        crate::agent_master::control(&context, &dir, &agent, &["set", &old.to_u32().to_string()])
            .await
            .unwrap();
    assert!(restored.contains("\"ok\":true"));
    assert!(scope.check(&context).await.is_ok());
    let runtime =
        crate::agents::register_with_context(&root, &args, direct.to_u32(), Some(context.clone()))
            .await
            .unwrap();
    crate::handlers::agent_master::handle(
        &parse(&[
            "agent-master",
            "codex",
            &next.to_u32().to_string(),
            "--data-path",
            root.to_str().unwrap(),
        ])
        .unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(
        crate::agent_master::current(&context, &dir, &agent)
            .await
            .unwrap()
            .contact,
        next
    );
    let raw = crate::agents::request(&root, "codex", "permissions list")
        .await
        .unwrap();
    let item = deltachat::webxdc::StatusUpdateItem {
        payload: raw.parse().unwrap(),
        ..Default::default()
    };
    assert_eq!(
        item.payload["owner_contact_id"].as_u64(),
        Some(next.to_u32().into())
    );
    drop(runtime);
    drop(context);
    drop(first);
    drop(second);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn audio_calls_are_codex_only_and_validate_signaling_without_media_dependencies() {
    assert!(crate::agent_calls::supported("codex", true));
    assert!(!crate::agent_calls::supported("grok", true));
    assert!(!crate::agent_calls::supported("codex", false));
    assert!(
        crate::agent_calls::validate_offer("v=0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 111\r\n", false)
            .is_ok()
    );
    assert!(crate::agent_calls::validate_offer("invalid", false).is_err());
    assert!(crate::agent_calls::validate_offer("v=0\nm=audio 9 RTP/AVP 0", true).is_err());
    let original = include_str!("../tests/fixtures/call-offer.sdp");
    let upstream = crate::agent_calls::audio_offer(original).unwrap();
    assert!(!upstream.contains("m=video"));
    assert!(!upstream.contains("m=application"));
    assert!(upstream.contains("a=group:BUNDLE 1\r\n"));
    let restored = crate::agent_calls::restore_answer(original, &upstream).unwrap();
    assert!(restored.contains("m=video 0"));
    assert!(restored.contains("m=application 0"));
    assert!(restored.find("m=audio").unwrap() < restored.find("m=application").unwrap());
    assert!(restored.find("m=video").unwrap() < restored.find("m=audio").unwrap());
    assert!(restored.contains("a=mid:0\r\na=inactive"));
    // Audio-only clients can still offer incompatible Delta data channels.
    let audio_and_data = original.split("m=video").next().unwrap().to_owned()
        + "m=audio"
        + original.split_once("m=audio").unwrap().1;
    let filtered = crate::agent_calls::audio_offer(&audio_and_data).unwrap();
    assert!(!filtered.contains("m=application"));
    assert!(
        crate::agent_calls::restore_answer(&audio_and_data, &filtered)
            .unwrap()
            .contains("m=application 0")
    );

    assert!(
        crate::agent_calls::restore_answer(
            original,
            "v=0\r\nm=audio 9 RTP/AVP 0\r\na=mid:wrong\r\n"
        )
        .is_err()
    );
    for argv in [
        vec!["agent-master", "codex"],
        vec!["agent-master", "1", "15"],
    ] {
        parse(&argv).unwrap().validate().unwrap();
    }
    assert!(
        parse(&["agent-master", "codex", "0"])
            .unwrap()
            .validate()
            .is_err()
    );
    assert!(crate::agent_guard::is_command("/master"));
    assert!(crate::agent_guard::is_command("/call"));
}

#[ignore = "live Codex SDP negotiation; uses installed provider authentication/network, no real audio"]
#[tokio::test(flavor = "multi_thread")]
async fn native_codex_call_negotiates_sdp_in_isolation() {
    use deltachat::contact::{self, ContactId};
    let root = std::path::PathBuf::from("/var/tmp")
        .join(format!("sigma-call-probe-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let context = offline_identity(&root, "call-bot").await;
    let person = offline_identity(&root, "call-master").await;
    let owner = contact::import_vcard(
        &context,
        &contact::make_vcard(&person, &[ContactId::SELF])
            .await
            .unwrap(),
    )
    .await
    .unwrap()[0];
    let chat = ChatId::create_for_contact(&context, owner).await.unwrap();
    let dir = root.join("agents/codex");
    std::fs::create_dir_all(&dir).unwrap();
    let offer = include_str!("../tests/fixtures/call-offer.sdp").to_owned();
    let id = context
        .place_outgoing_call(chat, offer.clone(), false)
        .await
        .unwrap();
    let scope = crate::agent_guard::Scope {
        dir: dir.clone(),
        owner,
        owner_chat: chat,
        chat,
        sender: owner,
    };
    let config = crate::agent_calls::Config {
        record: false,
        dir: dir.clone(),
        scope,
        id,
        offer,
        model: Some("gpt-6.1-sol".into()),
        custom: Some(
            "This is a signaling capability probe. No microphone or user audio is connected."
                .into(),
        ),
    };
    let executable = crate::handlers::agent_connect::executable("codex", None).unwrap();
    // An outgoing core fixture deliberately cannot be accepted as incoming. Reaching
    // that check proves the isolated Codex path supplied a valid remote SDP answer.
    let error = crate::agent_calls::run(&context, &executable, &config)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("call.is_incoming"), "{error}");
    assert!(
        crate::agent_sessions::load(&dir)
            .unwrap()
            .unwrap()
            .id
            .is_some()
    );
    // A second negotiation must resume the same thread through the metadata-only
    // response path, rather than creating a replacement conversation.
    let session = crate::agent_sessions::load(&dir).unwrap().unwrap();
    let error = crate::agent_calls::run(&context, &executable, &config)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("call.is_incoming"), "{error}");
    assert_eq!(crate::agent_sessions::load(&dir).unwrap().unwrap(), session);
    context.end_call(id).await.unwrap();
    drop(context);
    drop(person);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn outbound_call_cli_requires_explicit_native_codex_flag() {
    parse(&[
        "agent-connect",
        "codex",
        "--to",
        "me",
        "--allow-outbound-calls",
    ])
    .unwrap()
    .validate()
    .unwrap();
    for args in [
        vec![
            "agent-connect",
            "grok",
            "--to",
            "me",
            "--allow-outbound-calls",
        ],
        vec![
            "agent-connect",
            "codex",
            "--to",
            "me",
            "--allow-outbound-calls=true",
        ],
        vec!["agent-list", "--allow-outbound-calls"],
    ] {
        assert!(parse(&args).and_then(|a| a.validate()).is_err());
    }
    parse(&[
        "agent-act",
        "call",
        "message:15",
        "Ask when the meeting starts",
    ])
    .unwrap()
    .validate()
    .unwrap();
    assert!(
        parse(&["agent-act", "call", "15"])
            .unwrap()
            .validate()
            .is_err()
    );
}

#[ignore = "requires an installed Chromium; tests real local bidirectional WebRTC audio without provider credentials"]
#[tokio::test]
async fn outbound_browser_bridges_audio_both_directions() {
    let mut browser =
        crate::call_browser::Browser::start(&crate::call_browser::executable().unwrap())
            .await
            .unwrap();
    let (peer, provider) = browser.offers("[]").await.unwrap();
    assert!(peer.find("m=video").unwrap() < peer.find("m=audio").unwrap());
    assert!(!provider.contains("m=video"));
    let expression = format!(
        r#"(async()=>{{
        window.testPeers=[];
        const answer=async(sdp,frequency)=>{{
            const pc=new RTCPeerConnection();testPeers.push(pc);pc.streamIds=[];
            pc.ontrack=e=>{{pc.streamIds.push(e.streams[0]?.id);const audio=document.createElement('audio');audio.srcObject=new MediaStream([e.track]);audio.volume=0.001;audio.autoplay=true;document.body.appendChild(audio);audio.play();}};
            const oscillator=sigmaCall.ac.createOscillator();oscillator.frequency.value=frequency;
            const dest=sigmaCall.ac.createMediaStreamDestination();oscillator.connect(dest);oscillator.start();
            dest.stream.getTracks().forEach(t=>pc.addTrack(t,dest.stream));
            const ice=pc.createDataChannel('iceTrickling',{{negotiated:true,id:1}});
            const mute=pc.createDataChannel('mutedState',{{negotiated:true,id:3}});
            ice.onopen=()=>ice.send('null');mute.onopen=()=>mute.send(JSON.stringify({{videoEnabled:false,audioEnabled:true}}));
            await pc.setRemoteDescription({{type:'offer',sdp}});
            await pc.setLocalDescription(await pc.createAnswer());
            await new Promise(r=>setTimeout(r,1000));return pc.localDescription.sdp;
        }};
        return {{peer:await answer({},440),provider:await answer({},660)}};
    }})()"#,
        crate::output::encode(&peer.as_str().into()),
        crate::output::encode(&provider.as_str().into())
    );
    let answers = browser.eval(&expression).await.unwrap();
    browser
        .answer(false, answers.payload["peer"].as_str().unwrap())
        .await
        .unwrap();
    browser
        .answer(true, answers.payload["provider"].as_str().unwrap())
        .await
        .unwrap();
    let mut received = false;
    for _ in 0..10 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let stats=browser.eval("Promise.all(testPeers.map(async pc=>({state:pc.connectionState,audio:[...(await pc.getStats()).values()].filter(s=>s.type==='inbound-rtp'&&s.kind==='audio').map(s=>s.totalAudioEnergy)})))").await.unwrap();
        if stats.payload.as_array().unwrap().iter().all(|p| {
            p["state"] == "connected"
                && p["audio"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|e| e.as_f64().unwrap_or(0.0) > 0.0)
        }) {
            received = true;
            break;
        }
    }
    assert!(received, "both call legs must receive non-silent audio");
    browser.eval("sigmaCall.startRecording()").await.unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    browser.eval("sigmaCall.stopRecording()").await.unwrap();
    let recording = browser.eval("(async()=>{const blob=new Blob(sigmaCall.recordChunks,{type:'audio/webm'});const audio=await sigmaCall.ac.decodeAudioData(await blob.arrayBuffer());return {size:blob.size,duration:audio.duration,energy:audio.getChannelData(0).reduce((s,x)=>s+x*x,0)};})()").await.unwrap();
    assert!(recording.payload["size"].as_u64().unwrap() > 1000);
    assert!(recording.payload["duration"].as_f64().unwrap() > 1.0);
    assert!(recording.payload["energy"].as_f64().unwrap() > 0.01);
    let compatibility=browser.eval("(async()=>({streams:testPeers[0].streamIds,frames:[...(await testPeers[0].getStats()).values()].filter(s=>s.type==='inbound-rtp'&&s.kind==='video').reduce((n,s)=>n+(s.framesDecoded||0),0)}))()").await.unwrap();
    let streams = compatibility.payload["streams"].as_array().unwrap();
    assert_eq!(streams.len(), 2);
    assert!(
        streams[0].is_string() && streams[0] == streams[1],
        "audio and compatibility frames must belong to one remote stream"
    );
    assert!(
        compatibility.payload["frames"].as_u64().unwrap() > 0,
        "the recipient must receive real black frames, not an empty video section"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn outbound_tool_is_master_only_opt_in_and_idempotent() {
    use deltachat::contact::{self, ContactId};
    let root = socket_test_directory();
    std::fs::create_dir_all(&root).unwrap();
    let context = offline_identity(&root, "outbound-bot").await;
    let person = offline_identity(&root, "outbound-master").await;
    let owner = contact::import_vcard(
        &context,
        &contact::make_vcard(&person, &[ContactId::SELF])
            .await
            .unwrap(),
    )
    .await
    .unwrap()[0];
    let chat = ChatId::create_for_contact(&context, owner).await.unwrap();
    let dir = root.join("agents/codex");
    std::fs::create_dir_all(&dir).unwrap();
    let scope = crate::agent_guard::Scope {
        dir: dir.clone(),
        owner,
        owner_chat: chat,
        chat,
        sender: owner,
    };
    let server = crate::agent_actions::Server::start_for_sender(
        &context,
        chat,
        &dir,
        true,
        Some(scope.clone()),
    )
    .await
    .unwrap();
    let args = ["agent-act", "call", "15", "Ask about tomorrow's meeting"];
    let denied = agent_action_call(&server, &args, uuid::Uuid::new_v4(), None).await;
    assert_eq!(denied.payload["ok"], false);
    let (permission, mut rx) = crate::agent_outbound::register_permission(&dir).unwrap();
    assert!(server.instructions().contains("agent-act call TARGET"));
    let wrong = agent_action_call(&server, &args, uuid::Uuid::new_v4(), Some("wrong-key")).await;
    assert_eq!(wrong.payload["ok"], false);
    assert!(rx.try_recv().is_err());
    let id = uuid::Uuid::new_v4();
    let queued = agent_action_call(&server, &args, id, None).await;
    assert_eq!(queued.payload["ok"], true);
    assert!(rx.try_recv().is_ok());
    let replay = agent_action_call(&server, &args, id, None).await;
    assert_eq!(queued.payload, replay.payload);
    assert!(rx.try_recv().is_err());
    let mut guest = scope.clone();
    guest.sender = ContactId::SELF;
    assert!(
        crate::agent_outbound::queue(&context, Some(&guest), "15", "test")
            .await
            .is_err()
    );
    let mut group = scope.clone();
    group.chat = chat::create_group(&context, "group").await.unwrap();
    assert!(!crate::agent_outbound::enabled(&group));
    drop(permission);
    assert!(
        crate::agent_outbound::queue(&context, Some(&scope), "15", "test")
            .await
            .is_err()
    );
    // Cancellation remains available to the master without dialing permission.
    let ticket = crate::call_history::Ticket::new(&dir).unwrap();
    let cancel = ["agent-act", "call-cancel", ticket.id.as_str()];
    assert!(server.instructions().contains("agent-act call-cancel all"));
    let denied = agent_action_call(&server, &cancel, uuid::Uuid::new_v4(), Some("wrong-key")).await;
    assert_eq!(denied.payload["ok"], false);
    assert!(ticket.check().is_ok());
    let group_server = crate::agent_actions::Server::start_for_sender(
        &context,
        group.chat,
        &dir.join("group"),
        true,
        Some(group),
    )
    .await
    .unwrap();
    let denied = agent_action_call(&group_server, &cancel, uuid::Uuid::new_v4(), None).await;
    assert_eq!(denied.payload["ok"], false);
    assert!(ticket.check().is_ok());
    let guest_server = crate::agent_actions::Server::start_for_sender(
        &context,
        chat,
        &dir.join("guest"),
        false,
        Some(guest),
    )
    .await
    .unwrap();
    let denied = agent_action_call(&guest_server, &cancel, uuid::Uuid::new_v4(), None).await;
    assert_eq!(denied.payload["ok"], false);
    assert!(ticket.check().is_ok());
    let cancel_id = uuid::Uuid::new_v4();
    let result = agent_action_call(&server, &cancel, cancel_id, None).await;
    assert_eq!(result.payload["ok"], true);
    assert!(ticket.check().is_err());
    let replay = agent_action_call(&server, &cancel, cancel_id, None).await;
    assert_eq!(result.payload, replay.payload);
    drop(guest_server);
    drop(group_server);
    drop(ticket);
    drop(server);
    drop(context);
    drop(person);
    std::fs::remove_dir_all(root).unwrap();
}

#[ignore = "live Codex resume capability refresh; provider authentication/network required"]
#[tokio::test(flavor = "multi_thread")]
async fn native_codex_resume_refreshes_changed_capabilities() {
    let dir =
        std::env::temp_dir().join(format!("sigma-capability-refresh-{}", uuid::Uuid::new_v4()));
    let executable = crate::handlers::agent_connect::executable("codex", None).unwrap();
    let mut invocation = crate::agent_sessions::Invocation {
        executable: &executable,
        provider: "codex",
        model: Some("gpt-6.1-sol"),
        prompt: "The current runtime has messaging only. No outgoing call tool is enabled. Do not use tools for this test.",
        seconds: 120,
        dir: &dir,
        environment: &[],
        workspace: None,
    };
    invocation
        .run("Remember the word sigma-saffron. Can you call a contact with the current runtime?")
        .await
        .unwrap();
    let session = crate::agent_sessions::load(&dir).unwrap();
    invocation.prompt = "The runtime now explicitly enables outgoing calls. The tool command is: sigma agent-act call TARGET BRIEF. For this capability test, describe the command but do not execute tools or place any call.";
    let response=invocation.run("Can you call a contact now? State the command available to you and the remembered word. Do not call anyone.").await.unwrap();
    assert!(
        response.contains("agent-act call") && response.contains("sigma-saffron"),
        "{response}"
    );
    assert_eq!(crate::agent_sessions::load(&dir).unwrap(), session);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn call_contact_cards_allow_photos_without_inflating_identity() {
    let root = std::env::temp_dir().join(format!("sigma-vcard-size-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let file = root.join("contact.vcf");
    let data = format!(
        "BEGIN:VCARD\nVERSION:4.0\nFN:Person\nEMAIL:person@example.org\nKEY:test-key\nPHOTO:{}\n {}\nREV:20260101T000000Z\nEND:VCARD\n",
        "A".repeat(80000),
        "B".repeat(80000)
    );
    std::fs::write(&file, data).unwrap();
    let parsed = crate::agent_outbound::read_contact_card(&file).unwrap();
    assert!(
        parsed.contains("EMAIL:person@example.org")
            && parsed.contains("KEY:test-key")
            && parsed.contains("REV:")
    );
    assert!(!parsed.contains("PHOTO"));
    assert!(parsed.len() < 1024);
    std::fs::write(
        &file,
        format!("BEGIN:VCARD\nFN:{}\nEND:VCARD\n", "a".repeat(65536)),
    )
    .unwrap();
    assert!(crate::agent_outbound::read_contact_card(&file).is_err());
    std::fs::write(&file, "BEGIN:VCARD\nEND:VCARD\nBEGIN:VCARD\nEND:VCARD").unwrap();
    assert!(crate::agent_outbound::read_contact_card(&file).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn call_management_cli_validates_selectors_and_count() {
    for args in [
        vec!["agent-calls", "codex"],
        vec!["agent-calls", "1", "100"],
        vec!["agent-call-cancel", "codex", "all"],
        vec!["agent-call-cancel", "codex", "42"],
    ] {
        parse(&args).unwrap().validate().unwrap();
    }
    for args in [
        vec!["agent-calls"],
        vec!["agent-calls", "codex", "0"],
        vec!["agent-calls", "codex", "101"],
        vec!["agent-call-cancel", "codex", "wrong"],
        vec!["agent-call-cancel", "codex", "1"],
    ] {
        assert!(parse(&args).and_then(|a| a.validate()).is_err());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn call_history_lists_failures_and_core_calls_and_cancels() {
    use deltachat::contact::{self, ContactId};
    let root = std::env::temp_dir().join(format!("sigma-call-history-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let context = offline_identity(&root, "history-bot").await;
    let person = offline_identity(&root, "history-person").await;
    let contact = contact::import_vcard(
        &context,
        &contact::make_vcard(&person, &[ContactId::SELF])
            .await
            .unwrap(),
    )
    .await
    .unwrap()[0];
    let chat = ChatId::create_for_contact(&context, contact).await.unwrap();
    let dir = root.join("agents/codex");
    let failed = crate::call_history::Ticket::new(&dir).unwrap();
    failed
        .update("failed", None, Some("contact import failed"))
        .unwrap();
    let queued = crate::call_history::Ticket::new(&dir).unwrap();
    crate::call_history::control(&context, &dir, &["cancel", &queued.id])
        .await
        .unwrap();
    assert!(queued.check().is_err());
    let ticket = crate::call_history::Ticket::new(&dir).unwrap();
    let id = context
        .place_outgoing_call(
            chat,
            include_str!("../tests/fixtures/call-offer.sdp").into(),
            false,
        )
        .await
        .unwrap();
    ticket.update("ringing", Some(id), None).unwrap();
    ticket
        .diagnostics(
            "waiting-for-answer",
            Some(r#"{"peer":{"received":0,"sent":0}}"#),
        )
        .unwrap();
    let events = context.get_event_emitter();
    let scope = crate::agent_guard::Scope {
        dir: dir.clone(),
        owner: contact,
        owner_chat: chat,
        chat,
        sender: contact,
    };
    // The answer is broadcast only, exactly as core's receive path does it.
    // A different call's acceptance must not connect this call to the wrong peer.
    context.emit_event(deltachat::EventType::OutgoingCallAccepted {
        msg_id: deltachat::message::MsgId::new(id.to_u32() + 100),
        chat_id: chat,
        accept_call_info: "wrong call".into(),
    });
    context.emit_event(deltachat::EventType::OutgoingCallAccepted {
        msg_id: id,
        chat_id: chat,
        accept_call_info: "remote SDP answer".into(),
    });
    assert!(
        context
            .load_call_by_id(id)
            .await
            .unwrap()
            .unwrap()
            .accept_call_info
            .is_empty()
    );
    let answer = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        crate::agent_outbound::wait_answer(&context, &events, id, &scope, &ticket),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(answer.as_deref(), Some("remote SDP answer"));

    let raw = crate::call_history::control(&context, &dir, &["list", "50"])
        .await
        .unwrap();
    assert!(
        raw.contains("contact import failed")
            && raw.contains("duration_seconds")
            && raw.contains("ringing")
            && raw.contains("waiting-for-answer")
            && raw.contains("received")
    );
    crate::call_history::control(&context, &dir, &["cancel", &id.to_u32().to_string()])
        .await
        .unwrap();
    assert!(
        context
            .load_call_by_id(id)
            .await
            .unwrap()
            .unwrap()
            .is_ended()
    );
    crate::call_history::control(&context, &dir, &["cancel", &id.to_u32().to_string()])
        .await
        .unwrap();
    drop(ticket);
    drop(queued);
    drop(failed);
    let raw = crate::call_history::control(&context, &dir, &["list", "50"])
        .await
        .unwrap();
    assert!(raw.contains("interrupted"));
    drop(context);
    drop(person);
    std::fs::remove_dir_all(root).unwrap();
}

#[ignore = "live Codex authenticated cancellation; provider authentication/network required"]
#[tokio::test(flavor = "multi_thread")]
async fn native_codex_cancels_call_from_master_request() {
    use deltachat::contact::{self, ContactId};
    let root = std::env::temp_dir().join(format!("scc-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let context = offline_identity(&root, "cancel-bot").await;
    let person = offline_identity(&root, "cancel-master").await;
    let owner = contact::import_vcard(
        &context,
        &contact::make_vcard(&person, &[ContactId::SELF])
            .await
            .unwrap(),
    )
    .await
    .unwrap()[0];
    let chat = ChatId::create_for_contact(&context, owner).await.unwrap();
    let dir = root.join("agents/codex");
    let scope = crate::agent_guard::Scope {
        dir: dir.clone(),
        owner,
        owner_chat: chat,
        chat,
        sender: owner,
    };
    let mut server =
        crate::agent_actions::Server::start_for_sender(&context, chat, &dir, true, Some(scope))
            .await
            .unwrap();
    let binary = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/sigma");
    server
        .environment
        .iter_mut()
        .find(|(k, _)| k == "SIGMA_BIN")
        .unwrap()
        .1 = binary.to_string_lossy().into_owned();
    server.environment.push((
        "SIGMA_AGENT_SANDBOX_ROOT".into(),
        root.to_string_lossy().into_owned(),
    ));
    server.environment.push((
        "SIGMA_AGENT_STATE_DIR".into(),
        dir.to_string_lossy().into_owned(),
    ));
    let ticket = crate::call_history::Ticket::new(&dir).unwrap();
    let executable = crate::handlers::agent_connect::executable("codex", None).unwrap();
    let prompt = server.instructions();
    let invocation = crate::agent_sessions::Invocation {
        executable: &executable,
        provider: "codex",
        model: Some("gpt-6.1-sol"),
        prompt: &prompt,
        seconds: 120,
        dir: &dir,
        environment: &server.environment,
        workspace: Some(&server.outbox),
    };
    invocation
        .run("Hey, cancel the call please.")
        .await
        .unwrap();
    assert!(
        ticket.check().is_err(),
        "Codex must execute cancellation, not merely claim it"
    );
    drop(ticket);
    drop(server);
    drop(person);
    drop(context);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn voice_call_context_and_hangup_are_bound_to_the_current_thread() {
    let prompt = crate::agent_outbound::call_prompt(
        "Tell them they are amazing",
        "Recipient",
        "the bot master",
        Some("gpt-6.1-sol"),
    );
    let initial = crate::output::encode(&crate::agent_outbound::initial_context(&prompt));
    assert!(
        initial.contains("developer")
            && initial.contains("LIVE OUTGOING")
            && initial.contains("they are amazing")
            && initial.contains("the bot master")
            && initial.contains("gpt-6.1-sol")
    );
    let request = |thread: &str, name: &str, args: &str| {
        deltachat::webxdc::StatusUpdateItem {
        payload: format!(r#"{{"id":7,"method":"item/tool/call","params":{{"threadId":"{thread}","tool":"{name}","arguments":{args}}}}}"#).parse().unwrap(), ..Default::default()
    }
    };
    assert!(crate::agent_outbound::is_hangup_request(
        &request("current", "sigma_hang_up", "{}"),
        Some("current")
    ));
    for item in [
        request("other", "sigma_hang_up", "{}"),
        request("current", "shell", "{}"),
        request("current", "sigma_hang_up", r#"{"call_id":42}"#),
        request("current", "sigma_hang_up", "null"),
    ] {
        assert!(!crate::agent_outbound::is_hangup_request(
            &item,
            Some("current")
        ));
    }
    assert!(!crate::agent_outbound::is_hangup_request(
        &request("current", "sigma_hang_up", "{}"),
        None
    ));
}

#[ignore = "installed Chromium recording integration; no network/provider needed"]
#[tokio::test(flavor = "multi_thread")]
async fn call_recording_streams_saves_and_shares_only_with_authorized_master() {
    use deltachat::contact::{self, ContactId};
    let root = std::env::temp_dir().join(format!("scr-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let context = offline_identity(&root, "record-bot").await;
    let person = offline_identity(&root, "record-master").await;
    let owner = contact::import_vcard(
        &context,
        &contact::make_vcard(&person, &[ContactId::SELF])
            .await
            .unwrap(),
    )
    .await
    .unwrap()[0];
    let chat = ChatId::create_for_contact(&context, owner).await.unwrap();
    let dir = root.join("agents/codex");
    let scope = crate::agent_guard::Scope {
        dir: dir.clone(),
        owner,
        owner_chat: chat,
        chat,
        sender: owner,
    };
    let mut media = crate::call_recording::Media::new().await.unwrap();
    media.browser.offers("[]").await.unwrap();
    media
        .browser
        .eval("sigmaCall.nodes[1].gain.value=0.05")
        .await
        .unwrap();
    media
        .start(&scope, deltachat::message::MsgId::new(42))
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1300)).await;
    media.flush().await.unwrap();
    assert!(
        std::fs::metadata(dir.join("recordings/call-42.webm.part"))
            .unwrap()
            .len()
            > 100
    );
    media
        .finish(&context, &scope, deltachat::message::MsgId::new(42))
        .await
        .unwrap();
    assert!(dir.join("recordings/call-42.webm").is_file());
    assert!(!dir.join("recordings/call-42.webm.part").exists());
    let metadata = std::fs::read_to_string(dir.join("recordings/call-42.json")).unwrap();
    assert!(metadata.contains("queued-to-master"));
    let messages = chat::get_chat_msgs(&context, chat).await.unwrap().len();
    drop(media);
    let mut media = crate::call_recording::Media::new().await.unwrap();
    media.browser.offers("[]").await.unwrap();
    media
        .start(&scope, deltachat::message::MsgId::new(43))
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    let mut revoked = scope.clone();
    revoked.sender = ContactId::SELF;
    media
        .finish(&context, &revoked, deltachat::message::MsgId::new(43))
        .await
        .unwrap();
    assert!(dir.join("recordings/call-43.webm").is_file());
    assert_eq!(
        chat::get_chat_msgs(&context, chat).await.unwrap().len(),
        messages
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(dir.join("recordings/call-42.webm"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    drop(media);
    drop(context);
    drop(person);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn recording_requires_native_codex_and_explicit_flag() {
    parse(&["agent-connect", "codex", "--to", "me", "--record-calls"])
        .unwrap()
        .validate()
        .unwrap();
    for args in [
        vec!["agent-connect", "grok", "--to", "me", "--record-calls"],
        vec![
            "agent-connect",
            "codex",
            "--to",
            "me",
            "--record-calls",
            "--exec",
            "custom",
        ],
    ] {
        assert!(parse(&args).unwrap().validate().is_err());
    }
}

#[ignore = "installed Chromium incoming recording transport; no provider/network needed"]
#[tokio::test(flavor = "multi_thread")]
async fn recording_bridge_answers_incoming_calls_with_duplex_audio() {
    let mut browser =
        crate::call_browser::Browser::start(&crate::call_browser::executable().unwrap())
            .await
            .unwrap();
    let caller=browser.eval(r#"(async()=>{
      window.caller=new RTCPeerConnection({bundlePolicy:'max-bundle'});
      window.audio=new AudioContext();await audio.resume();
      window.testNodes=[];
      window.play=e=>{const a=document.createElement('audio');a.srcObject=new MediaStream([e.track]);a.volume=.001;a.autoplay=true;document.body.appendChild(a);a.play();};caller.ontrack=play;
      const dst=audio.createMediaStreamDestination(),osc=audio.createOscillator();osc.frequency.value=440;osc.connect(dst);osc.start();testNodes.push(dst,osc);
      caller.addTransceiver('video',{direction:'sendrecv'});caller.addTrack(dst.stream.getAudioTracks()[0],dst.stream);
      caller.createDataChannel('iceTrickling',{negotiated:true,id:1});caller.createDataChannel('mutedState',{negotiated:true,id:3});
      await caller.setLocalDescription(await caller.createOffer());await new Promise(r=>setTimeout(r,1000));return caller.localDescription.sdp;
    })()"#).await.unwrap();
    let (answer, offer) = browser
        .incoming("[]", caller.payload.as_str().unwrap())
        .await
        .unwrap();
    let provider=browser.eval(&format!(r#"(async()=>{{
      await caller.setRemoteDescription({{type:'answer',sdp:{}}});
      window.providerTest=new RTCPeerConnection();providerTest.ontrack=play;
      const dst=audio.createMediaStreamDestination(),osc=audio.createOscillator();osc.frequency.value=660;osc.connect(dst);osc.start();testNodes.push(dst,osc);
      providerTest.addTrack(dst.stream.getAudioTracks()[0],dst.stream);
      await providerTest.setRemoteDescription({{type:'offer',sdp:{}}});
      await providerTest.setLocalDescription(await providerTest.createAnswer());await new Promise(r=>setTimeout(r,1000));return providerTest.localDescription.sdp;
    }})()"#,crate::output::encode(&answer.into()),crate::output::encode(&offer.into()))).await.unwrap();
    browser
        .answer(true, provider.payload.as_str().unwrap())
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let result=browser.eval("Promise.all([caller,providerTest].map(async pc=>[...(await pc.getStats()).values()].filter(s=>s.type==='inbound-rtp'&&s.kind==='audio').some(s=>s.totalAudioEnergy>0)))").await.unwrap();
    assert_eq!(result.payload.to_string(), "[true,true]");
    let error = browser.eval("sigmaCall.error").await.unwrap();
    assert!(error.payload.is_null(), "{}", error.payload);
}
