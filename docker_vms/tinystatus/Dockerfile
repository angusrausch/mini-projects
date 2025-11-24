FROM python:3-alpine
WORKDIR /usr/src/app

# Setup Tinystatus

COPY history.html.theme \
     index.html.theme \
     requirements.txt \
     tinystatus.py ./

RUN pip install --no-cache-dir -r requirements.txt

# Setup NGINX

RUN apk update && apk add nginx
RUN mkdir /etc/nginx/ssl && \
    /bin/rm -f /etc/nginx/http.d/*

# Setup Keepalived

RUN apk add keepalived
RUN mkdir -p /etc/keepalived

CMD sh -c " keepalived -f /etc/keepalived/keepalived.conf & nginx -g 'daemon off;' & python3 /usr/src/app/tinystatus.py"
