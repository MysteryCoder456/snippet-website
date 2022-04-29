use rocket::fs::TempFile;

#[derive(FromForm)]
pub struct RegisterForm<'a> {
    #[field(validate = len(2..40).or_else(msg!("Username must be between 2 and 40 characters long")))]
    pub username: &'a str,

    #[field(validate = contains('@').or_else(msg!("Invalid email address")))]
    pub email: &'a str,

    #[field(validate = len(6..).or_else(msg!("Password must be at least 6 characters long")))]
    pub password: &'a str,

    #[field(validate = eq(self.password).or_else(msg!("Passwords do not match")))]
    pub confirm_password: &'a str,
}

#[derive(FromForm)]
pub struct LoginForm<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(FromForm)]
pub struct AddSnippetForm<'a> {
    #[field(validate = len(2..=100).or_else(msg!("Title must be between 2 and 100 characters long")))]
    pub title: &'a str,

    #[field(validate = len(..=20).or_else(msg!("Language cannot be more than 20 characters long")))]
    pub language: &'a str,

    pub code: &'a str,
}

#[derive(FromForm)]
pub struct EditProfileFrom<'a> {
    #[field(validate = len(5..=200).or_else(msg!("Bio must be between 5 and 200 characters long")))]
    pub bio: &'a str,

    #[field(validate = len(2..=25).or_else(msg!("Occupation must be between 2 and 25 characters long")))]
    pub occupation: &'a str,

    pub avatar: TempFile<'a>,
}
