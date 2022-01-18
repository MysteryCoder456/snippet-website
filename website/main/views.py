from django.shortcuts import render, redirect
from django.http import HttpRequest
from django.contrib import messages
from django.contrib.auth.decorators import login_required
from .models import CodeSnippet
from .forms import AddSnippetForm

# Create your views here.


def home(request: HttpRequest):
    snippets = list(CodeSnippet.objects.all())
    snippets.sort(key=lambda x: x.date_created, reverse=True)
    return render(request, "main/home.html", {"code_snippets": snippets})


def snippet(request: HttpRequest, snippet_id: int):
    return "Coming soon..."


@login_required
def add_snippet(request: HttpRequest):
    form = AddSnippetForm()

    if request.method == "POST":
        form = AddSnippetForm(request.POST)

        if form.is_valid():
            title = form.cleaned_data["title"]
            language = form.cleaned_data["language"]
            code = form.cleaned_data["code"]
            request.user.codesnippet_set.create(
                title=title, language=language, code=code
            )
            messages.success(request, "New snippet was created successfully!")
            return redirect("home")

    return render(request, "main/add_snippet.html", {"form": form})
