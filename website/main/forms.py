from django import forms
from .models import CodeSnippet


class AddSnippetForm(forms.ModelForm):
    class Meta:
        model = CodeSnippet
        fields = ["title", "language", "code"]
