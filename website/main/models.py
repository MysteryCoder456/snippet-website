from django.db import models
from django.contrib.auth.models import User

# Create your models here.


class CodeSnippet(models.Model):
    date_created = models.DateTimeField(auto_now_add=True)
    language = models.CharField(max_length=10, null=False, default="plaintext")
    title = models.CharField(max_length=70, null=False)
    code = models.TextField(null=False, default="Hello World!")
    author = models.ForeignKey(User, on_delete=models.CASCADE)

    def __str__(self):
        return f"ID={self.id} Author={self.author.username}"
