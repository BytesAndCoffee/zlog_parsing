"""Out-of-band Telegram notifications that do not depend on MySQL."""

import json
import logging
import os
import urllib.error
import urllib.parse
import urllib.request


def send_telegram(message: str) -> bool:
    """Send one operational alert without logging the bot credential."""
    token = os.getenv("TELEGRAM_BOT_TOKEN")
    chat_id = os.getenv("TELEGRAM_CHAT_ID")
    if not token or not chat_id:
        logging.error("Telegram operational alert credentials are not configured")
        return False

    request = urllib.request.Request(
        "https://api.telegram.org/bot{}/sendMessage".format(token),
        data=urllib.parse.urlencode({"chat_id": chat_id, "text": message}).encode(),
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            payload = json.loads(response.read())
            return response.status == 200 and bool(payload.get("ok"))
    except (urllib.error.URLError, TimeoutError, ValueError) as exc:
        # Exception strings can contain the request URL and bot token.
        logging.error("Telegram operational alert failed: %s", type(exc).__name__)
        return False
