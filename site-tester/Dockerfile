FROM python:3.12

RUN pip install httpx

WORKDIR /app
COPY ./site-tester.py /app

ENTRYPOINT [ "python", "/app/site-tester.py" ]
