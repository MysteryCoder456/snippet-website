#[derive(FromForm)]
pub struct RegisterForm<'a> {
    #[field(validate = len(2..))]
    pub username: &'a str,

    #[field(validate = contains('@').or_else(msg!("Invalid email address")))]
    pub email: &'a str,

    #[field(validate = len(6..))]
    #[field(validate = eq(self.confirm_password).or_else(msg!("Passwords do not match")))]
    pub password: &'a str,

    #[field(validate = eq(self.password).or_else(msg!("Passwords do not match")))]
    pub confirm_password: &'a str,
}
