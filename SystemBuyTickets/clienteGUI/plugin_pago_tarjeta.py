# plugin_pago_tarjeta.py

def procesar_pago():
    from PyQt5.QtWidgets import QDialog, QLabel, QLineEdit, QPushButton, QVBoxLayout, QMessageBox
    import random

    class PagoPluginTarjeta(QDialog):
        def __init__(self):
            super().__init__()
            self.setWindowTitle("Pago con Tarjeta")
            self.init_ui()
            self.resultado = None

        def init_ui(self):
            layout = QVBoxLayout()
            layout.addWidget(QLabel("Ingrese los datos de su tarjeta:"))
            self.txt_numero = QLineEdit()
            self.txt_numero.setPlaceholderText("Número de Tarjeta")
            layout.addWidget(self.txt_numero)
            self.txt_fecha = QLineEdit()
            self.txt_fecha.setPlaceholderText("Fecha de Expiración")
            layout.addWidget(self.txt_fecha)
            self.txt_cvv = QLineEdit()
            self.txt_cvv.setPlaceholderText("CVV")
            layout.addWidget(self.txt_cvv)
            self.btn_pagar = QPushButton("Pagar")
            self.btn_pagar.clicked.connect(self.realizar_pago)
            layout.addWidget(self.btn_pagar)
            self.btn_cancelar = QPushButton("Cancelar Compra")
            self.btn_cancelar.clicked.connect(self.cancelar_pago)
            layout.addWidget(self.btn_cancelar)
            self.setLayout(layout)

        def realizar_pago(self):
            if random.choice([True, False]):
                QMessageBox.information(self, "Pago Aprobado", "Su pago ha sido aprobado.")
                self.resultado = True
            else:
                QMessageBox.warning(self, "Pago Rechazado", "Su pago ha sido rechazado.")
                self.resultado = False
            self.accept()

        def cancelar_pago(self):
            self.resultado = None
            self.reject()

    dialogo = PagoPluginTarjeta()
    dialogo.exec_()
    return dialogo.resultado
