from django.urls import path
from . import views

urlpatterns = [
    path("", views.home, name="home"),
    path("snippet/<int:snippet_id>", views.snippet, name="snippet"),
    path("snippet/add/", views.add_snippet, name="add_snippet"),
]
