use aws_config::BehaviorVersion;
use aws_sdk_sesv2::types::{Destination, EmailContent, Template};

pub async fn send_reset_password_email(
    email: &str,
    reset_link: &str,
) -> Result<(), aws_sdk_sesv2::Error> {
    let config = aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await;

    let client = aws_sdk_sesv2::Client::new(&config);

    let mut dest: Destination = Destination::builder().build();
    dest.to_addresses = Some(vec![String::from(email)]);

    let email_template = Template::builder()
        .template_name("ResetPasswordTemplate_PTBR")
        .template_data(String::from(format!(
            r#"{{"resetPasswordLink":"{}"}}"#,
            reset_link
        )))
        .build();

    let email_content = EmailContent::builder().template(email_template).build();

    client
        .send_email()
        .from_email_address("no-reply@dev.finly.digital")
        .destination(dest)
        .content(email_content)
        .send()
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_send_email() {
        match send_reset_password_email("elissonlima0@gmail.com", "https://g1.globo.com").await {
            Ok(_) => {
                println!("It worked!");
            }
            Err(e) => {
                println!("error! {}", e);
            }
        }
    }
}
