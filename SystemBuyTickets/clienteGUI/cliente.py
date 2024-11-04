# cliente.py

import sys
import socket
import threading
import json
from PyQt5.QtWidgets import (
    QApplication, QMainWindow, QLabel, QPushButton,
    QComboBox, QSpinBox, QVBoxLayout, QHBoxLayout,
    QWidget, QGridLayout, QMessageBox, QScrollArea
)
from PyQt5.QtCore import QTimer, Qt, pyqtSignal, QObject

# Clase Asiento
class Asiento(QPushButton):
    def __init__(self, asiento_info):
        super().__init__(str(asiento_info['asiento']))
        self.asiento_info = asiento_info
        self.estado = asiento_info['estado']
        print(f"Creando Asiento: Zona {asiento_info['zona']}, Fila {asiento_info['fila']}, Asiento {asiento_info['asiento']}, Estado {self.estado}")
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

# Clase VentanaAsientos
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
        print(f"Datos de respuesta recibidos: {datos_respuesta}")
        try:
            self.init_ui()
            print("VentanaAsientos inicializada correctamente.")
        except Exception as e:
            print("Excepción en VentanaAsientos.__init__:", e)

    def init_ui(self):
        v_layout = QVBoxLayout()

        categoria = self.datos_respuesta['categoria']
        mensaje = self.datos_respuesta['mensaje']

        label_info = QLabel(f"Categoría: {categoria}\n{mensaje}")
        v_layout.addWidget(label_info)

        # Área de scroll para los asientos
        scroll_area = QScrollArea()
        scroll_widget = QWidget()
        self.asientos_layout = QGridLayout(scroll_widget)
        self.mostrar_asientos()
        scroll_area.setWidgetResizable(True)
        scroll_area.setWidget(scroll_widget)
        v_layout.addWidget(scroll_area)

        h_layout_botones = QHBoxLayout()
        self.btn_confirmar = QPushButton('Confirmar Compra')
        self.btn_confirmar.clicked.connect(self.confirmar_compra)
        self.btn_cancelar = QPushButton('Cancelar Compra')
        self.btn_cancelar.clicked.connect(self.cancelar_compra)
        h_layout_botones.addWidget(self.btn_confirmar)
        h_layout_botones.addWidget(self.btn_cancelar)
        v_layout.addLayout(h_layout_botones)

        self.setLayout(v_layout)

        # Temporizador para liberar reservas
        self.temporizador_reserva = QTimer()
        self.temporizador_reserva.setInterval(120000)  # 2 minutos en milisegundos
        self.temporizador_reserva.timeout.connect(self.cancelar_compra_por_tiempo)
        self.temporizador_reserva.start()

    def mostrar_asientos(self):
        print("Mostrando asientos en VentanaAsientos...")
        try:
            self.botones_asientos = {}
            self.asientos_categoria = self.datos_respuesta['asientos_categoria']
            self.asientos_recomendados = self.datos_respuesta['asientos_recomendados']
            print(f"Cantidad de asientos en la categoría: {len(self.asientos_categoria)}")
            print(f"Asientos recomendados: {self.asientos_recomendados}")

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
                    # Hacer una copia del asiento_info para no modificar el original
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

    def confirmar_compra(self):
        self.temporizador_reserva.stop()
        asientos_seleccionados = []
        for asiento in self.botones_asientos.values():
            if asiento.estado == 'Recomendado':
                asiento_info = {
                    'zona': asiento.asiento_info['zona'],
                    'fila': asiento.asiento_info['fila'],
                    'asiento': asiento.asiento_info['asiento'],
                    # No enviamos 'estado' al servidor
                }
                asientos_seleccionados.append(asiento_info)

        if not asientos_seleccionados:
            QMessageBox.warning(self, 'Advertencia', 'No ha seleccionado ningún asiento para comprar.')
            return

        solicitud = {
            'indice_categoria': self.solicitud['indice_categoria'],
            'cantidad_boletos': len(asientos_seleccionados),
            'confirmar_compra': True,
            'asientos_recomendados': asientos_seleccionados
        }
        threading.Thread(target=self.enviar_solicitud, args=(solicitud, True)).start()

        # Deshabilitar botones
        self.btn_confirmar.setEnabled(False)
        self.btn_cancelar.setEnabled(False)

    def cancelar_compra(self):
        self.temporizador_reserva.stop()
        asientos_seleccionados = []
        for asiento in self.botones_asientos.values():
            if asiento.estado == 'Recomendado':
                asiento_info = {
                    'zona': asiento.asiento_info['zona'],
                    'fila': asiento.asiento_info['fila'],
                    'asiento': asiento.asiento_info['asiento'],
                    # No enviamos 'estado' al servidor
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
        self.btn_confirmar.setEnabled(False)
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
                print("Respuesta del servidor (confirmar/cancelar):", respuesta)
                self.respuesta = respuesta
                self.es_confirmacion = es_confirmacion
                self.actualizar_interfaz_signal.emit()
        except Exception as e:
            QMessageBox.critical(self, 'Error', f'Error de conexión: {e}')

    def actualizar_interfaz(self):
        print("actualizar_interfaz called (VentanaAsientos)")
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
    actualizar_interfaz_signal = pyqtSignal()

    def __init__(self):
        super().__init__()
        self.setWindowTitle('Sistema de Compra de Entradas')
        self.actualizar_interfaz_signal.connect(self.actualizar_interfaz)
        self.init_ui()

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

        # Layouts
        h_layout = QHBoxLayout()
        h_layout.addWidget(QLabel('Categoría:'))
        h_layout.addWidget(self.combo_categoria)
        h_layout.addWidget(QLabel('Cantidad de Entradas:'))
        h_layout.addWidget(self.spin_boletos)
        h_layout.addWidget(self.btn_buscar)

        v_layout = QVBoxLayout()
        v_layout.addLayout(h_layout)
        v_layout.addWidget(self.label_respuesta)

        central_widget = QWidget()
        central_widget.setLayout(v_layout)
        self.setCentralWidget(central_widget)

    def buscar_asientos(self):
        indice_categoria = self.combo_categoria.currentIndex()
        cantidad_boletos = self.spin_boletos.value()

        solicitud = {
            'indice_categoria': indice_categoria,
            'cantidad_boletos': cantidad_boletos,
            'confirmar_compra': False,
            'asientos_recomendados': None
        }
        threading.Thread(target=self.enviar_solicitud, args=(solicitud,)).start()

    def enviar_solicitud(self, solicitud):
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                s.connect(('127.0.0.1', 7878))
                s.sendall(json.dumps(solicitud).encode())
                respuesta = s.recv(65536).decode()
                print("Respuesta del servidor:", respuesta)
                self.respuesta = respuesta
                self.solicitud = solicitud
                self.actualizar_interfaz_signal.emit()
        except Exception as e:
            self.label_respuesta.setText(f'Error de conexión: {e}')

    def actualizar_interfaz(self):
        print("actualizar_interfaz called (Cliente)")
        try:
            print("Procesando respuesta en Cliente...")
            datos = json.loads(self.respuesta)
            print("Datos recibidos:", datos)
            categoria = datos['categoria']
            mensaje = datos['mensaje']
            asientos_categoria = datos['asientos_categoria']
            asientos_recomendados = datos['asientos_recomendados']

            if asientos_categoria:
                # Abrir la ventana de asientos
                print("Abriendo ventana de asientos...")
                self.ventana_asientos = VentanaAsientos(datos, self.solicitud)
                self.ventana_asientos.show()
            else:
                self.label_respuesta.setText('No hay asientos disponibles en esta categoría.')
        except json.JSONDecodeError:
            print("Error al decodificar JSON")
            self.label_respuesta.setText('Error al procesar la respuesta del servidor.')
        except Exception as e:
            print("Excepción en actualizar_interfaz (Cliente):", e)
            self.label_respuesta.setText(f'Error al procesar la respuesta: {e}')

if __name__ == '__main__':
    app = QApplication(sys.argv)
    cliente = Cliente()
    cliente.show()
    sys.exit(app.exec_())
