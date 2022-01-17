from django.db import models
from django.contrib.auth.models import User

# Create your models here.


class Profile(models.Model):
    user = models.OneToOneField(User, on_delete=models.CASCADE)
    bio = models.CharField(max_length=200, default="Hi there! I like coding.")
    occupation = models.CharField(max_length=25, default="Cool Coder")
    image = models.ImageField(
        default="profile_pics/default.png", upload_to="profile_pics"
    )

    def __str__(self):
        return f"ID={self.id} Username={self.user.username}"
