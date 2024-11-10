# cliente.py

import sys
import socket
import threading
import json
import importlib
from PyQt5.QtWidgets import (
    QApplication, QMainWindow, QLabel, QPushButton,
    QComboBox, QSpinBox, QVBoxLayout, QHBoxLayout,
    QWidget, QGridLayout, QMessageBox, QScrollArea
)
from PyQt5.QtCore import QTimer, Qt, pyqtSignal

# Clase Asiento permanece igual
class Asiento(QPushButton):
    def __init__(self, asiento_info):
        super().__init__(str(asiento_info['asiento']))
        self.asiento_info = asiento_info
        self.estado = asiento_info['estado']
        self.setCheckable(True)
        self.actualizar_color()
        self.clicked.connect(self.cambiar_seleccion)

    def actualizar_color(self):
        if self.estado == 'Disponible':
            self.setStyleSheet('background-color: gray')
        elif self.estado == 'Reservada':
            self.setStyleSheet('background-color: yellow')
            self.setEnabled(False)
        elif self.estado == 'Comprada':
            self.setStyleSheet('background-color: red')
            self.setEnabled(False)
        elif self.estado == 'Recomendado':
            self.setStyleSheet('background-color: blue')

    def cambiar_seleccion(self):
        if self.isChecked():
            self.setStyleSheet('background-color: blue')
            self.estado = 'Recomendado'
        else:
            self.setStyleSheet('background-color: gray')
            self.estado = 'Disponible'

# Clase VentanaAsientos permanece igual
class VentanaAsientos(QWidget):
    actualizar_interfaz_signal = pyqtSignal()

    def __init__(self, datos_respuesta, solicitud):
        super().__init__()
        self.setWindowTitle('Seleccionar Asientos')
        self.actualizar_interfaz_signal.connect(self.actualizar_interfaz)
        self.datos_respuesta = datos_respuesta
        self.solicitud = solicitud
        self.asientos_categoria = []
        self.asientos_recomendados = []
        self.botones_asientos = {}
        self.init_ui()

    def init_ui(self):
        v_layout = QVBoxLayout()

        categoria = self.datos_respuesta['categoria']
        mensaje = self.datos_respuesta['mensaje']

        label_info = QLabel(f"Categoría: {categoria}\n{mensaje}")
        v_layout.addWidget(label_info)

        # Leyenda de colores
        leyenda_layout = QHBoxLayout()
        leyendas = [
            ('Disponible', 'gray'),
            ('Reservada', 'yellow'),
            ('Comprada', 'red'),
            ('Recomendado', 'blue')
        ]
        for texto, color in leyendas:
            label_color = QLabel()
            label_color.setFixedSize(20, 20)
            label_color.setStyleSheet(f'background-color: {color}')
            leyenda_layout.addWidget(label_color)
            leyenda_layout.addWidget(QLabel(texto))
        v_layout.addLayout(leyenda_layout)

        # Selección del método de pago
        metodo_pago_layout = QHBoxLayout()
        metodo_pago_layout.addWidget(QLabel('Método de Pago:'))
        self.combo_metodo_pago = QComboBox()
        self.combo_metodo_pago.addItems(['Tarjeta', 'PayPal', 'Criptomonedas'])
        metodo_pago_layout.addWidget(self.combo_metodo_pago)
        v_layout.addLayout(metodo_pago_layout)

        # Área de scroll para los asientos
        scroll_area = QScrollArea()
        scroll_widget = QWidget()
        self.asientos_layout = QGridLayout(scroll_widget)
        self.mostrar_asientos()
        scroll_area.setWidgetResizable(True)
        scroll_area.setWidget(scroll_widget)
        v_layout.addWidget(scroll_area)

        h_layout_botones = QHBoxLayout()
        self.btn_realizar_compra = QPushButton('Realizar Compra')
        self.btn_realizar_compra.clicked.connect(self.realizar_compra)
        self.btn_cancelar = QPushButton('Cancelar Compra')
        self.btn_cancelar.clicked.connect(self.cancelar_compra)
        h_layout_botones.addWidget(self.btn_realizar_compra)
        h_layout_botones.addWidget(self.btn_cancelar)
        v_layout.addLayout(h_layout_botones)

        self.setLayout(v_layout)

        # Temporizador para liberar reservas
        self.temporizador_reserva = QTimer()
        self.temporizador_reserva.setInterval(120000)  # 2 minutos en milisegundos
        self.temporizador_reserva.timeout.connect(self.cancelar_compra_por_tiempo)
        self.temporizador_reserva.start()

    def mostrar_asientos(self):
        try:
            self.botones_asientos = {}
            self.asientos_categoria = self.datos_respuesta['asientos_categoria']
            self.asientos_recomendados = self.datos_respuesta['asientos_recomendados']

            # Crear un conjunto de claves de asientos recomendados para identificarlos
            asientos_recomendados_set = {(a['zona'], a['fila'], a['asiento']) for a in self.asientos_recomendados}

            # Crear un diccionario de asientos, clave: (zona, fila, asiento), valor: asiento_info
            asientos_dict = {}
            for asiento_info in self.asientos_categoria:
                clave_asiento = (asiento_info['zona'], asiento_info['fila'], asiento_info['asiento'])
                asientos_dict[clave_asiento] = asiento_info

            # Actualizar el estado de los asientos recomendados a 'Recomendado'
            for clave_asiento in asientos_recomendados_set:
                if clave_asiento in asientos_dict:
                    asiento_info = asientos_dict[clave_asiento].copy()
                    asiento_info['estado'] = 'Recomendado'
                    asientos_dict[clave_asiento] = asiento_info

            # Agrupar asientos por zona y fila
            zonas = {}
            for asiento_info in asientos_dict.values():
                zona_nombre = asiento_info['zona']
                fila_numero = asiento_info['fila']
                if zona_nombre not in zonas:
                    zonas[zona_nombre] = {}
                if fila_numero not in zonas[zona_nombre]:
                    zonas[zona_nombre][fila_numero] = []
                zonas[zona_nombre][fila_numero].append(asiento_info)

            # Ordenar las zonas
            zonas_ordenadas = dict(sorted(zonas.items()))

            row = 0  # Fila actual en el grid layout

            for zona_nombre, filas in zonas_ordenadas.items():
                # Agregar etiqueta de zona
                label_zona = QLabel(f"--- {zona_nombre} ---")
                label_zona.setAlignment(Qt.AlignCenter)
                self.asientos_layout.addWidget(label_zona, row, 0, 1, 20)  # Ocupa 20 columnas
                row += 1

                # Ordenar las filas
                filas_ordenadas = dict(sorted(filas.items()))
                for fila_numero, asientos in filas_ordenadas.items():
                    # Agregar etiqueta de fila
                    label_fila = QLabel(f"Fila {fila_numero}")
                    label_fila.setAlignment(Qt.AlignLeft)
                    self.asientos_layout.addWidget(label_fila, row, 0)
                    col = 1  # Columna actual en el grid layout

                    # Ordenar los asientos por número
                    asientos_ordenados = sorted(asientos, key=lambda x: x['asiento'])
                    for asiento_info in asientos_ordenados:
                        clave_asiento = (asiento_info['zona'], asiento_info['fila'], asiento_info['asiento'])
                        asiento = Asiento(asiento_info)
                        self.asientos_layout.addWidget(asiento, row, col)
                        self.botones_asientos[clave_asiento] = asiento
                        col += 1
                    row += 1  # Siguiente fila para la próxima fila de asientos

                # Espacio entre zonas
                row += 1
        except Exception as e:
            print("Excepción en mostrar_asientos:", e)

    def realizar_compra(self):
        asientos_seleccionados = []
        for asiento in self.botones_asientos.values():
            if asiento.estado == 'Recomendado':
                asiento_info = {
                    'zona': asiento.asiento_info['zona'],
                    'fila': asiento.asiento_info['fila'],
                    'asiento': asiento.asiento_info['asiento'],
                }
                asientos_seleccionados.append(asiento_info)

        if not asientos_seleccionados:
            QMessageBox.warning(self, 'Advertencia', 'No ha seleccionado ningún asiento para comprar.')
            return

        # Obtener el método de pago seleccionado
        metodo_pago = self.combo_metodo_pago.currentText()
        plugin_name = ''
        if metodo_pago == 'Tarjeta':
            plugin_name = 'plugin_pago_tarjeta'
        elif metodo_pago == 'PayPal':
            plugin_name = 'plugin_pago_paypal'
        elif metodo_pago == 'Criptomonedas':
            plugin_name = 'plugin_pago_criptomonedas'
        else:
            QMessageBox.warning(self, 'Error', 'Método de pago no soportado.')
            return

        # Iniciar proceso de pago
        resultado_pago = self.procesar_pago(plugin_name)
        if resultado_pago is None:
            # El usuario canceló el pago
            QMessageBox.information(self, 'Pago Cancelado', 'El pago ha sido cancelado.')
            return
        elif resultado_pago:
            # Pago aprobado, confirmar compra
            self.temporizador_reserva.stop()
            solicitud = {
                'indice_categoria': self.solicitud['indice_categoria'],
                'cantidad_boletos': len(asientos_seleccionados),
                'confirmar_compra': True,
                'asientos_recomendados': asientos_seleccionados
            }
            threading.Thread(target=self.enviar_solicitud, args=(solicitud, True)).start()
            # Deshabilitar botones
            self.btn_realizar_compra.setEnabled(False)
            self.btn_cancelar.setEnabled(False)
        else:
            # Pago rechazado
            QMessageBox.warning(self, 'Pago Rechazado', 'El pago no fue aprobado. Intente nuevamente.')

    def procesar_pago(self, plugin_name):
        try:
            plugin = importlib.import_module(plugin_name)
            resultado_pago = plugin.procesar_pago()
            return resultado_pago
        except Exception as e:
            print(f"Error al cargar el plugin {plugin_name}: {e}")
            QMessageBox.critical(self, 'Error', f'Error al procesar el pago: {e}')
            return False

    def cancelar_compra(self):
        self.temporizador_reserva.stop()
        asientos_seleccionados = []
        for asiento in self.botones_asientos.values():
            if asiento.estado == 'Recomendado':
                asiento_info = {
                    'zona': asiento.asiento_info['zona'],
                    'fila': asiento.asiento_info['fila'],
                    'asiento': asiento.asiento_info['asiento'],
                }
                asientos_seleccionados.append(asiento_info)

        solicitud = {
            'indice_categoria': self.solicitud['indice_categoria'],
            'cantidad_boletos': len(asientos_seleccionados),
            'confirmar_compra': False,
            'asientos_recomendados': asientos_seleccionados
        }
        threading.Thread(target=self.enviar_solicitud, args=(solicitud, True)).start()

        # Deshabilitar botones
        self.btn_realizar_compra.setEnabled(False)
        self.btn_cancelar.setEnabled(False)

    def cancelar_compra_por_tiempo(self):
        self.cancelar_compra()
        QMessageBox.information(self, 'Información', 'Tiempo de reserva expirado. Los asientos han sido liberados.')
        self.close()

    def enviar_solicitud(self, solicitud, es_confirmacion):
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                s.connect(('127.0.0.1', 7878))
                s.sendall(json.dumps(solicitud).encode())
                respuesta = s.recv(65536).decode()
                self.respuesta = respuesta
                self.es_confirmacion = es_confirmacion
                self.actualizar_interfaz_signal.emit()
        except Exception as e:
            QMessageBox.critical(self, 'Error', f'Error de conexión: {e}')

    def actualizar_interfaz(self):
        try:
            datos = json.loads(self.respuesta)
            mensaje = datos['mensaje']
            QMessageBox.information(self, 'Respuesta del Servidor', mensaje)

            if self.es_confirmacion:
                self.close()
        except json.JSONDecodeError:
            QMessageBox.critical(self, 'Error', 'Error al procesar la respuesta del servidor.')
        except Exception as e:
            print("Excepción en actualizar_interfaz (VentanaAsientos):", e)

# Clase principal del cliente
class Cliente(QMainWindow):
    actualizar_interfaz_signal = pyqtSignal(int)

    def __init__(self):
        super().__init__()
        self.setWindowTitle('Sistema de Compra de Entradas')
        self.actualizar_interfaz_signal.connect(self.actualizar_interfaz)
        self.init_ui()
        self.responses = {}
        self.responses_lock = threading.Lock()
        self.ventanas_asientos = {}

    def init_ui(self):
        # Componentes de la interfaz
        self.combo_categoria = QComboBox()
        self.combo_categoria.addItems([
            'Platea Este', 'Platea Oeste', 'General Norte', 'General Sur'
        ])

        self.spin_boletos = QSpinBox()
        self.spin_boletos.setRange(1, 10)

        self.btn_buscar = QPushButton('Buscar Asientos')
        self.btn_buscar.clicked.connect(self.buscar_asientos)

        self.label_respuesta = QLabel('')

        # Representación del estadio
        self.btn_cargar_estadio = QPushButton('Mostrar Representación del Estadio')
        self.btn_cargar_estadio.clicked.connect(self.cargar_estadio)
        self.estadio_widget = QWidget()
        self.estadio_layout = QGridLayout(self.estadio_widget)

        # Layouts
        h_layout = QHBoxLayout()
        h_layout.addWidget(QLabel('Categoría:'))
        h_layout.addWidget(self.combo_categoria)
        h_layout.addWidget(QLabel('Cantidad de Entradas:'))
        h_layout.addWidget(self.spin_boletos)
        h_layout.addWidget(self.btn_buscar)

        v_layout = QVBoxLayout()
        v_layout.addLayout(h_layout)
        v_layout.addWidget(self.btn_cargar_estadio)
        v_layout.addWidget(self.estadio_widget)
        v_layout.addWidget(self.label_respuesta)

        central_widget = QWidget()
        central_widget.setLayout(v_layout)
        self.setCentralWidget(central_widget)

    def cargar_estadio(self):
        # (El código de esta función permanece igual)
        # ...

        # Limpiar el layout anterior
        for i in reversed(range(self.estadio_layout.count())):
            widget_to_remove = self.estadio_layout.itemAt(i).widget()
            self.estadio_layout.removeWidget(widget_to_remove)
            widget_to_remove.setParent(None)

        # Crear la representación simplificada del estadio
        # Usando una cuadrícula de 3x3 para posicionar las categorías
        # Las posiciones son relativas y representativas

        # Mapa de posiciones: {(fila, columna): ('Nombre de la categoría', 'Color')}
        mapa_estadio = {
            (0, 1): ('General Norte', '#ADD8E6'),   # Azul Claro
            (1, 0): ('Platea Oeste', '#C0C0C0'),    # Plata
            (1, 1): ('Campo de Juego', '#008000'),  # Verde
            (1, 2): ('Platea Este', '#FFD700'),     # Oro
            (2, 1): ('General Sur', '#CD7F32'),     # Bronce
        }

        for posicion, (nombre_categoria, color_categoria) in mapa_estadio.items():
            fila, columna = posicion
            if nombre_categoria == 'Campo de Juego':
                label = QLabel(nombre_categoria)
                label.setAlignment(Qt.AlignCenter)
                label.setStyleSheet(f'background-color: {color_categoria}; border: 1px solid black;')
                self.estadio_layout.addWidget(label, fila, columna)
            else:
                boton_categoria = QPushButton(nombre_categoria)
                boton_categoria.setFixedSize(100, 50)
                boton_categoria.setStyleSheet(f'background-color: {color_categoria}; border: 1px solid black;')
                boton_categoria.setEnabled(False)
                self.estadio_layout.addWidget(boton_categoria, fila, columna)

        # Ajustar estiramientos para centrar el contenido
        self.estadio_layout.setRowStretch(0, 1)
        self.estadio_layout.setRowStretch(1, 1)
        self.estadio_layout.setRowStretch(2, 1)
        self.estadio_layout.setColumnStretch(0, 1)
        self.estadio_layout.setColumnStretch(1, 1)
        self.estadio_layout.setColumnStretch(2, 1)

    def buscar_asientos(self):
        indice_categoria = self.combo_categoria.currentIndex()
        cantidad_boletos = self.spin_boletos.value()

        solicitud = {
            'indice_categoria': indice_categoria,
            'cantidad_boletos': cantidad_boletos,
            'confirmar_compra': False,
            'asientos_recomendados': None
        }

        # Enviar 3 solicitudes idénticas al servidor de forma concurrente
        for i in range(3):
            threading.Thread(target=self.enviar_solicitud, args=(solicitud.copy(), i)).start()

    def enviar_solicitud(self, solicitud, identifier):
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                s.connect(('127.0.0.1', 7878))
                s.sendall(json.dumps(solicitud).encode())
                respuesta = s.recv(65536).decode()
                # Guardar la respuesta con el identificador
                with self.responses_lock:
                    self.responses[identifier] = (respuesta, solicitud)
                # Emitir señal para actualizar la interfaz
                self.actualizar_interfaz_signal.emit(identifier)
        except Exception as e:
            # Comunicamos el error a la interfaz principal
            self.label_respuesta.setText(f'Error de conexión: {e}')

    def actualizar_interfaz(self, identifier):
        try:
            with self.responses_lock:
                data = self.responses.pop(identifier, None)
            if data is None:
                return
            respuesta, solicitud = data
            datos = json.loads(respuesta)
            categoria = datos['categoria']
            mensaje = datos['mensaje']
            asientos_categoria = datos['asientos_categoria']
            asientos_recomendados = datos['asientos_recomendados']

            if asientos_categoria:
                # Abrir la ventana de asientos
                ventana_asientos = VentanaAsientos(datos, solicitud)
                ventana_asientos.show()
                # Guardar la referencia a la ventana para evitar que sea recolectada por el recolector de basura
                self.ventanas_asientos[identifier] = ventana_asientos
            else:
                self.label_respuesta.setText('No hay asientos disponibles en esta categoría.')
        except json.JSONDecodeError:
            self.label_respuesta.setText('Error al procesar la respuesta del servidor.')
        except Exception as e:
            self.label_respuesta.setText(f'Error al procesar la respuesta: {e}')

if __name__ == '__main__':
    app = QApplication(sys.argv)
    cliente = Cliente()
    cliente.show()
    sys.exit(app.exec_())
