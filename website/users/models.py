from django.db import models
from django.contrib.auth.models import User
from PIL import Image

# Create your models here.


class Profile(models.Model):
    default_image_name = "profile_pics/default.png"

    user = models.OneToOneField(User, on_delete=models.CASCADE)
    bio = models.CharField(max_length=200, default="Hi there! I like coding.")
    occupation = models.CharField(max_length=25, default="Cool Coder")
    image = models.ImageField(
        default=default_image_name, upload_to="profile_pics"
    )

    def __str__(self):
        return f"ID={self.id} Username={self.user.username}"

    def save(self):
        super().save()
        img = Image.open(self.image.path)

        # Crop the image into a square
        final_size = min(*img.size)
        crop_rect = (
            (img.size[0] - final_size) // 2,
            (img.size[1] - final_size) // 2,
            (img.size[0] + final_size) // 2,
            (img.size[1] + final_size) // 2,
        )
        img = img.crop(crop_rect)

        img.save(self.image.path)
