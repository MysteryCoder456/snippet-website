from django.contrib.auth.views import LoginView, LogoutView
from django.urls import path
from . import views


urlpatterns = [
    path(
        "login/",
        LoginView.as_view(template_name="users/login.html"),
        name="login",
    ),
    path(
        "logout/",
        LogoutView.as_view(template_name="users/logout.html"),
        name="logout",
    ),
    path("register/", views.register, name="register"),
    path("profile/<int:user_id>", views.profile, name="profile"),
    path("profile/edit/", views.edit_profile, name="edit_profile"),
]
