# Agrupador de subtítulos por posición
Esta utilidad permite unir 2 subtítulos en formato srt en un único archivo que mostrará un subtítulo en la parte superior y el otro en la inferior.

## Importante
Para mostrar los carteles de los subtítulos se utilizan especificaciones que no pertenecen al formato srt. Sin embargo, reproductores de vídeo como [VLC](https://www.videolan.org) las aceptan y las procesan correctamente. Por lo tanto, es importante tener en cuenta que el **subtítulo que se genera no cumple las especificaciones del formato** y podría no mostrarse correctamente dependiendo del reproductor utilizado.

## Uso
```bash
Usage: unir_subtitulos_arriba_abajo --superior <SUPERIOR> --inferior <INFERIOR>

Options:
  -s, --superior <SUPERIOR>  ruta que apunta al archivo que se situará en la parte superior
  -i, --inferior <INFERIOR>  ruta que apunta al archivo que se situará en la parte inferior
  -h, --help                 Print help
  -V, --version              Print version
  ```


