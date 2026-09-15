# Use an official Python runtime as a parent image
FROM python:3.12

# Set the working directory in the container to /app
WORKDIR /app

# Add the current directory contents into the container at /app
COPY . /app

# Install runtime dependencies and the package itself.
RUN pip install --no-cache-dir -r requirements.txt && \
    pip install --no-cache-dir --no-deps .

# Make the main.sh script executable
RUN chmod +x main.sh

HEALTHCHECK --interval=1m --timeout=10s --start-period=30s --retries=3 \
    CMD ["python3", "-m", "zlog_parsing.healthcheck"]

# Run the application when the container launches
CMD ["./main.sh"]
