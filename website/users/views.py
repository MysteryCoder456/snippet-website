import os
from django.shortcuts import render, redirect
from django.contrib import messages
from django.contrib.auth.models import User
from django.contrib.auth.decorators import login_required
from django.http import HttpRequest
from .forms import ProfileUpdateForm, UserRegisterForm
from .models import Profile

# Create your views here.


def register(request: HttpRequest):
    form = UserRegisterForm()

    if request.method == "POST":
        form = UserRegisterForm(request.POST)

        if form.is_valid():
            form.save()
            username = form.cleaned_data["username"]
            messages.success(
                request,
                f"Welcome {username}! Your account has been created.",
            )
            return redirect("login")

    return render(request, "users/register.html", {"form": form})


def profile(request: HttpRequest, user_id: int):
    user = User.objects.get(id=user_id)

    if user.codesnippet_set.exists():
        first_snippet = user.codesnippet_set.first()
        latest_snippet = user.codesnippet_set.latest("date_created")
    else:
        first_snippet = latest_snippet = None

    return render(
        request,
        "users/profile.html",
        {
            "requested_user": user,
            "first_snippet": first_snippet,
            "latest_snippet": latest_snippet,
        },
    )


@login_required
def edit_profile(request: HttpRequest):
    form = ProfileUpdateForm(instance=request.user.profile)

    if request.method == "POST":
        previous_image = request.user.profile.image
        form = ProfileUpdateForm(
            request.POST, request.FILES, instance=request.user.profile
        )

        if form.is_valid():
            form.save()

            # Delete previous image
            if previous_image.name != Profile.default_image_name:
                os.remove(previous_image.path)

            messages.success(request, "Profile updated successfully!")
            return redirect("profile", user_id=request.user.id)

    return render(request, "users/edit_profile.html", {"form": form})
