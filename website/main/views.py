from django.shortcuts import render
from django.http import HttpRequest
from .models import CodeSnippet

# Create your views here.


def home(request: HttpRequest):
    snippets = list(CodeSnippet.objects.all())
    snippets.sort(key=lambda x: x.date_created, reverse=True)
    return render(request, "main/home.html", {"code_snippets": snippets})
