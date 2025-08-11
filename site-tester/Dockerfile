FROM python:3.12

RUN pip install requests httpx

WORKDIR /app
COPY ./test_site.py /app

ENTRYPOINT [ "python", "/app/test_site.py" ]
