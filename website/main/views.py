from django.shortcuts import render
from .models import CodeSnippet

# Create your views here.


def home(request):
    snippets = list(CodeSnippet.objects.all())
    snippets.sort(key=lambda x: x.date_created, reverse=True)
    return render(request, "home.html", {"code_snippets": snippets})
