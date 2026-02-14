# Use an official Python runtime as a parent image
FROM python:3.13-slim

# Set the working directory in the container
WORKDIR /app

# Copy the project files
COPY pyproject.toml README.md main.py ./

# Install the application and its dependencies
RUN pip install --no-cache-dir .

# Expose the port the application listens on
EXPOSE 8080

# Run the application
CMD ["python", "main.py"]
