# 🏦 bcv-scraper

Aplicación simple que hace _web scraping_ a la [página del Banco Central de Venezuela](https://www.bcv.org.ve/) dado un intérvalo y expone los cambios del dólar de referencia en JSON a través de una REST API.

# Ejecución

-   Clone el repositorio.
-   Cree un archivo de configuración en el directorio raíz del proyecto usando el archivo `config.yml.example` de referencia.
-   Ejecute usando `cargo`.

```
cargo run
```

-   Para compilar la aplicación, ejecute:

```
cargo build
```

-   Para crear una versión preparada para lanzamiento, ejecute:

```
cargo build --release
```

-   Para ejecutar el binario compilado directamente, es necesario colocar el archivo de configuración en el mismo directorio del binario, o en el directorio de trabajo.

# Endpoint - `GET /api/v1/rates`

Responde un estado HTTP `200 OK` con un JSON del siguiente formato:

```
{
    "rates": {
        // Todos los cambios están expresados en Bs.
        "eur": 0.0,
        "cny": 0.0,
        "try": 0.0,
        "rub": 0.0,
        "usd": 0.0
    },
    // Última vez que la caché del servicio fue actualizada en formato de tiempo Unix
    "updatedAt": 1758351872383
}
```
