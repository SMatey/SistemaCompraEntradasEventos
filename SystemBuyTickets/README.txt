Proyecto realizado por los estudiantes:
Sebastian Matey Rodriguez, carnet: 2022178169
Jeison Blanco

Link del video/documentacion:
https://youtu.be/uyTRjGgbe8E

Puntos que faltaron por mencionar en el video.
1. Implementacion de Sockets y procesos en paralelo:
	-El programa utiliza sockets TCP para la comunicación entre el cliente y el servidor. Para manejar múltiples solicitudes al mismo 
	tiempo, se emplea Tokio, un runtime asincrónico que permite ejecutar tareas en paralelo sin necesidad de gestionar hilos directamente.

	-Paralelismo: Cada solicitud del cliente se ejecuta en su propia tarea asincrónica usando tokio::spawn. 
	Tokio gestiona las tareas en segundo plano utilizando un pool de hilos, permitiendo que se ejecuten concurrentemente sin bloqueo.

	-Sincronización: No es necesario sincronizar hilos manualmente, ya que Tokio se encarga de manejar la concurrencia de manera segura. 
	Cada tarea es independiente, eliminando la necesidad de sincronización compleja.

2. Mejoras a gran escala para una implementacion mas robusta
	-Mejorar el manejo de errores
	-Mejorar el entorno en cuanto a seguridad
	-Optimizar los recursos para evitar saturacion


====================================================================================================================
------------------------Creamos el procedimiento almacenado
CREATE OR REPLACE FUNCTION control_acceso_atributos(
    p_tabla TEXT,          -- Nombre de la tabla
    p_insertar TEXT[],     -- Atributos permitidos para inserción
    p_modificar TEXT[],    -- Atributos permitidos para modificación
    p_eliminar TEXT[]      -- Atributos permitidos para eliminación
)
RETURNS TEXT               -- Retorna el nombre de la vista creada
LANGUAGE plpgsql
AS $$
DECLARE
    v_atributos_vista TEXT := '';  -- Lista de atributos para la vista
    v_atributo RECORD;             -- Variable para almacenar cada atributo de la tabla
    v_vista_nombre TEXT;           -- Nombre de la vista
BEGIN
    -- Validación: Verificar que los atributos existen en la tabla usando information_schema
    FOR v_atributo IN
        SELECT column_name
        FROM information_schema.columns
        WHERE table_name = p_tabla
    LOOP
        IF v_atributo.column_name = ANY(p_insertar)
        OR v_atributo.column_name = ANY(p_modificar)
        OR v_atributo.column_name = ANY(p_eliminar) THEN
            v_atributos_vista := v_atributos_vista || v_atributo.column_name || ', ';
        END IF;
    END LOOP;

    -- Si no hay atributos válidos, lanzar un error
    IF v_atributos_vista = '' THEN
        RAISE EXCEPTION 'No se especificaron atributos válidos para la vista.';
    END IF;

    -- Eliminar la última coma y espacio
    v_atributos_vista := substring(v_atributos_vista, 1, length(v_atributos_vista) - 2);

    -- Definir el nombre de la vista
    v_vista_nombre := p_tabla || '_vista_acceso';

    -- Crear la vista con los atributos permitidos
    EXECUTE 'CREATE OR REPLACE VIEW ' || v_vista_nombre || ' AS SELECT ' || v_atributos_vista || ' FROM ' || p_tabla;

    -- Crear el trigger para controlar las operaciones en la vista
    PERFORM agregar_trigger_a_vista(v_vista_nombre, p_insertar, p_modificar, p_eliminar);

    -- Retornar el nombre de la vista creada
    RETURN v_vista_nombre;
END;
$$;

-----------------------------------Creamos la funcion del trigger
CREATE OR REPLACE FUNCTION control_acceso_trigger_func()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    p_insertar TEXT[];
    p_modificar TEXT[];
    p_eliminar TEXT[];
    attr TEXT;  -- Variable para iterar sobre los nombres de los atributos
    attr_value TEXT; -- Variable para almacenar el valor del atributo
BEGIN
    -- Asignar los argumentos del trigger a las variables
    p_insertar := TG_ARGV[0]::TEXT[];  -- Atributos permitidos para inserción
    p_modificar := TG_ARGV[1]::TEXT[]; -- Atributos permitidos para modificación
    p_eliminar := TG_ARGV[2]::TEXT[];  -- Atributos permitidos para eliminación

    -- Validación para INSERT
    IF (TG_OP = 'INSERT') THEN
        -- Iterar sobre los atributos insertados
        FOR attr IN SELECT jsonb_object_keys(row_to_json(NEW)::jsonb) LOOP
            -- Obtener el valor del atributo
            EXECUTE format('SELECT $1.%I', attr) INTO attr_value USING NEW;
            
            -- Validar si el valor no es NULL y no está permitido en la lista de inserción
            IF attr_value IS NOT NULL AND NOT attr = ANY(p_insertar) THEN
                RAISE EXCEPTION 'No se permite la inserción en el atributo %', attr;
            END IF;
        END LOOP;
    END IF;

    -- Validación para UPDATE
    IF (TG_OP = 'UPDATE') THEN
        FOR attr IN SELECT jsonb_object_keys(row_to_json(NEW)::jsonb) LOOP
            EXECUTE format('SELECT $1.%I', attr) INTO attr_value USING NEW;
            IF attr_value IS NOT NULL AND NOT attr = ANY(p_modificar) THEN
                RAISE EXCEPTION 'No se permite la modificación del atributo %', attr;
            END IF;
        END LOOP;
    END IF;

    -- Validación para DELETE
    IF (TG_OP = 'DELETE') THEN
        IF array_length(p_eliminar, 1) = 0 THEN
            RAISE EXCEPTION 'No se permite la eliminación en esta vista.';
        ELSE
            -- Realizar la eliminación en la tabla original
            EXECUTE 'DELETE FROM ' || TG_TABLE_NAME || ' WHERE id = ' || quote_literal(OLD.id);
        END IF;
    END IF;

    RETURN NULL; -- Permitir la operación si pasa la validación
END;
$$;

--Agregamos el trigger a la vista
CREATE OR REPLACE FUNCTION agregar_trigger_a_vista(
    p_vista TEXT,        -- Nombre de la vista
    p_insertar TEXT[],   -- Atributos permitidos para inserción
    p_modificar TEXT[],  -- Atributos permitidos para modificación
    p_eliminar TEXT[]    -- Atributos permitidos para eliminación
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    -- Crear el trigger con INSTEAD OF y pasar los arreglos correctamente utilizando format()
    EXECUTE format(
        'CREATE TRIGGER %I_trigger
         INSTEAD OF INSERT OR UPDATE OR DELETE ON %I
         FOR EACH ROW EXECUTE FUNCTION control_acceso_trigger_func(%L, %L, %L);',
        p_vista, p_vista, p_insertar, p_modificar, p_eliminar
    );
END;
$$;


---------------------------------Realizamos pruebas
--Tablas de prueba:
CREATE TABLE clientes (
    id SERIAL PRIMARY KEY,
    nombre VARCHAR(100),
    email VARCHAR(100),
    telefono VARCHAR(15),
    direccion TEXT
);

INSERT INTO clientes (nombre, email, telefono, direccion) 
VALUES 
('Juan Perez', 'juan.perez@example.com', '555-1234', 'Calle Falsa 123'),
('Maria Garcia', 'maria.garcia@example.com', '555-5678', 'Avenida Siempreviva 456');

CREATE TABLE productos (
    id SERIAL PRIMARY KEY,
    nombre VARCHAR(100),
    precio NUMERIC(10, 2),
    stock INT,
    categoria VARCHAR(50)
);

INSERT INTO productos (nombre, precio, stock, categoria) 
VALUES 
('Laptop', 1200.00, 10, 'Electrónica'),
('Smartphone', 800.00, 20, 'Electrónica'),
('Cafetera', 150.00, 15, 'Hogar');

CREATE TABLE empleados (
    id SERIAL PRIMARY KEY,
    nombre VARCHAR(100),
    puesto VARCHAR(100),
    salario NUMERIC(10, 2),
    departamento VARCHAR(50)
);

INSERT INTO empleados (nombre, puesto, salario, departamento) 
VALUES 
('Carlos Lopez', 'Gerente', 3000.00, 'Administración'),
('Ana Torres', 'Asistente', 1500.00, 'Recursos Humanos'),
('Luis Rivera', 'Desarrollador', 2500.00, 'Tecnología');

-------------------------------------Pruebas tabla clientes
-- Crear la vista y asociar el trigger con atributos permitidos para inserción, modificación y eliminación
SELECT control_acceso_atributos(
    'clientes',
    ARRAY['nombre', 'email'],  -- Atributos para inserción
    ARRAY['telefono'],         -- Atributos para modificación
    ARRAY['id']                -- Atributos para eliminación
);

SELECT * FROM clientes;
-- Probar inserciones válidas en la vista
INSERT INTO clientes_vista_acceso (nombre, email) VALUES ('Pedro', 'pedro@example.com');

-- Probar una modificación no permitida (esto debe lanzar un error)
UPDATE clientes_vista_acceso SET email = 'nuevo.email@example.com' WHERE id = 1;

-- Probar una modificación permitida (esto debería funcionar)
UPDATE clientes_vista_acceso SET telefono = '555-9999' WHERE id = 1;

-- Probar una eliminación permitida (esto debería funcionar)
DELETE FROM clientes_vista_acceso WHERE id = 2;

------------------------------Pruebas tabla productos
-- Crear la vista y asociar el trigger para la tabla productos
SELECT control_acceso_atributos(
    'productos',
    ARRAY['nombre', 'precio'],   -- Atributos para inserción
    ARRAY['precio'],             -- Atributos para modificación
    ARRAY[]::TEXT[]              -- No se permite eliminación
);

-- Probar inserciones válidas en la vista
INSERT INTO productos_vista_acceso (nombre, precio) VALUES ('Teclado', 50.00);

-- Intentar eliminar un producto (esto debe lanzar un error)
DELETE FROM productos_vista_acceso WHERE id = 1;

-- Intentar modificar un atributo no permitido (esto debe lanzar un error)
UPDATE productos_vista_acceso SET stock = 50 WHERE id = 1;

-- Modificar un atributo permitido
UPDATE productos_vista_acceso SET precio = 1100.00 WHERE id = 1;

------------------------------Pruebas tabla empleados
-- Crear la vista y asociar el trigger para la tabla empleados
SELECT control_acceso_atributos(
    'empleados',
    ARRAY['nombre', 'puesto'],   -- Atributos para inserción
    ARRAY['salario'],            -- Atributos para modificación
    ARRAY['id']                  -- Atributos para eliminación
);

-- Intentar insertar un empleado válido
INSERT INTO empleados_vista_acceso (nombre, puesto) VALUES ('Daniela Diaz', 'Contadora');

-- Intentar modificar un atributo no permitido (esto debe lanzar un error)
UPDATE empleados_vista_acceso SET departamento = 'Finanzas' WHERE id = 1;

-- Intentar modificar un atributo permitido
UPDATE empleados_vista_acceso SET salario = 3200.00 WHERE id = 2;

-- Intentar eliminar un empleado permitido
DELETE FROM empleados_vista_acceso WHERE id = 3;



