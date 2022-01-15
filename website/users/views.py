from django.shortcuts import render, redirect
from django.contrib import messages
from .forms import UserRegisterForm

# Create your views here.


def login(request):
    ...


def register(request):
    if request.method == "POST":
        form = UserRegisterForm(request.POST)

        if form.is_valid():
            form.save()
            username = form.cleaned_data["username"]
            messages.success(
                request,
                f"Welcome {username}! Your account has been created.",
            )
            return redirect("home")

    else:
        form = UserRegisterForm()

    return render(request, "users/register.html", {"form": form})
