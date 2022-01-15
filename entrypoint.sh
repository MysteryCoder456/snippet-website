#!/bin/sh

python manage.py migrate --no-input
python manage.py collectstatic --no-input

gunicorn -b :8080 --certfile ssl/cert.pem --keyfile ssl/cert.key website.wsgi

