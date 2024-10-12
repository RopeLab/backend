use mail_send::mail_builder::MessageBuilder;
use mail_send::SmtpClientBuilder;
use crate::error::APIResult;
use crate::markdown_files::{expect_content_populated, get_file_content, get_mail_file_meta_data, populate_mail_file_with_url, populate_mail_file_with_user_data};
use crate::user_data::UserData;

pub async fn send_mail(to_name: &str, to_mail: &str, subject: &str, content: &str) -> APIResult<()> {
    
    let html =  markdown::to_html(content);
    
    let message = MessageBuilder::new()
        .from(("Stroby", "stroby@ropelab.de"))
        .to(vec![(to_name, to_mail)])
        .subject(subject)
        .html_body(html)
        .text_body(content);
    
    SmtpClientBuilder::new("betelgeuse.uberspace.de", 587)
        .implicit_tls(false)
        .credentials(("stroby@ropelab.de", "e50UAytMQNMdZtQNdBF0paJ5ZD"))
        .connect()
        .await
        .unwrap()
        .send(message)
        .await
        .unwrap();

    Ok(())
}

pub async fn send_password_reset_mail(email: &str, user_data: UserData, url: &str) -> APIResult<()> {
    let content = get_file_content("/mails/new_password.md")?;
    let (meta, content) = get_mail_file_meta_data(content)?;

    let content = populate_mail_file_with_user_data(content, &user_data);
    let content = populate_mail_file_with_url(content, url);
    expect_content_populated(&content)?;
    
    send_mail(&user_data.name, email, &meta.title, &content).await?;
    
    Ok(())
}