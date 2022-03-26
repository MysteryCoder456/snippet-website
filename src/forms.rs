#[derive(FromForm)]
pub struct RegisterForm<'a> {
    #[field(validate = len(2..).or_else(msg!("Username must be at least 2 characters long")))]
    pub username: &'a str,

    #[field(validate = contains('@').or_else(msg!("Invalid email address")))]
    pub email: &'a str,

    #[field(validate = len(6..).or_else(msg!("Password must be at least 6 characters long")))]
    pub password: &'a str,

    #[field(validate = eq(self.password).or_else(msg!("Passwords do not match")))]
    pub confirm_password: &'a str,
}
