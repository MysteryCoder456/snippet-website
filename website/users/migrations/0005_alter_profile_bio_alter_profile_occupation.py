# Generated by Django 4.0.1 on 2022-01-17 14:52

from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('users', '0004_alter_profile_image'),
    ]

    operations = [
        migrations.AlterField(
            model_name='profile',
            name='bio',
            field=models.CharField(default='Hi there! I like coding.', max_length=200),
        ),
        migrations.AlterField(
            model_name='profile',
            name='occupation',
            field=models.CharField(default='Cool Coder', max_length=25),
        ),
    ]
