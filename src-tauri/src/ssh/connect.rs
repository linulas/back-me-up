use openssh::{KnownHosts, Session};

pub async fn to_home_server() -> Result<(), openssh::Error> {
    let session = Session::connect("ssh://linus@192.168.50.178:777", KnownHosts::Strict).await?;

    let ls = session.command("ls").output().await?;
    eprintln!(
        "{}",
        String::from_utf8(ls.stdout).expect("server output was not valid UTF-8")
    );

    let whoami = session.command("whoami").output().await?;
    assert_eq!(whoami.stdout, b"me\n");

    session.close().await?;
    Ok(())
}
